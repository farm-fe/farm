"""Record actual release build/public-consumer outcomes in a fixed main workflow."""
# cspell:words argparse importlib npmjs urllib
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import urllib.parse

spec = importlib.util.spec_from_file_location("release_record_entry", Path(__file__).with_name("incremental-release-entry.py"))
entry = importlib.util.module_from_spec(spec); spec.loader.exec_module(entry)
check = entry.check


def current_job(transport, expected):
    run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
    run = transport.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
    check(run["id"] == run_id and run["run_attempt"] == attempt and run["head_sha"] == os.environ["GITHUB_SHA"]
          and run["path"] == entry.WORKFLOW and run["status"] == "in_progress", "Actual release recording identity differs")
    rows = [row for row in entry.execution.jobs_for(transport, run) if row["name"] == expected and row["status"] == "in_progress"]
    check(len(rows) == 1, "Exact running release producer job required")
    return rows[0]


def build_record(manifest, source, job, candidate):
    check(manifest["sourceSHA"] == source["sha"] and manifest["sourceTree"] == source["tree"], "Build source differs")
    receipt = json.loads((Path(candidate) / "receipt.json").read_text(encoding="utf-8"))
    check(receipt["status"] == "complete-seal" and receipt["testedCommit"] == source["sha"] and receipt["testedTree"] == source["tree"], "Independent candidate seal incomplete")
    for row in manifest["packages"]:
        entry.strict_archive(Path(candidate) / "archives", row)
    return {"kind": "release-seal", "profile": "release", "namespace": "farm-release-v1",
            "manifestHash": entry.chain.digest(manifest), "producerJobId": job["id"], "sourceTree": source["tree"],
            "packageInputs": entry.evidence.package_input_hashes(source)}


def check_archives(manifest, directory, source_sha):
    """Validate frozen bytes only; no consumer, scratch or test result is emitted."""
    check(manifest.get('sourceSHA') == source_sha, 'Archive source differs')
    check(manifest.get('chain') == 'release' and manifest.get('namespace') == 'farm-release-v1', 'Archive release namespace differs')
    rows = manifest.get('packages')
    check(isinstance(rows, list) and rows, 'Archive inventory missing')
    check(len({row['kind'] + ':' + row['name'] for row in rows}) == len(rows), 'Duplicate archive identity')
    for row in rows:
        entry.strict_archive(Path(directory) / 'archives', row)


def public_record(manifest, reports, job, registry):
    check(not entry.archive_integrity_mode(manifest), "Legacy public reports require the original consumer policy")
    expected = {"ubuntu-24.04", "macos-latest", "windows-latest"}
    check(set(reports) == expected, "All three public consumer reports required")
    for report in reports.values():
        check(report.get("status") == "passed" and report.get("publicRegistry") is True
              and report.get("workspacePatches") is False and report.get("cleanupConfirmed") is not False,
              "Public registry consumer or cleanup failed")
        check(report.get('kind') != 'release-byte-integrity-v1', 'Legacy consumer reports cannot use byte-only proof')
        required={"npm-install", "esm-cjs", "types", "docs-build"} | (set() if manifest.get("publicationScope") == "npm" else {"sdk"})
        check(required.issubset(report.get("stages", [])), "Actual public consumer stages missing")
    return registry_record(manifest, job, registry, integrity=False)


def registry_record(manifest, job, registry, *, integrity=True):
    """Registry bytes and stable tag read-back, without installation or tests."""
    with registry.budget(1500):
        return _registry_record(manifest, job, registry, integrity=integrity)


def _registry_record(manifest, job, registry, *, integrity):
    check(entry.archive_integrity_mode(manifest) == integrity, "Frozen manifest and registry proof policy differ")
    records = {row["kind"] + ":" + row["name"]: row for row in manifest["packages"]}
    actual = registry.verify(records)
    check(actual == {key: row["archiveSha256"] for key, row in records.items()}, "Independent public registry bytes differ")
    for row in records.values():
        if row["kind"] == "npm":
            latest = registry.json("https://registry.npmjs.org/" + urllib.parse.quote(row["name"], safe="") + "/latest")
            check(latest.get("name") == row["name"], "Public latest package identity differs")
            def version(value):
                check(isinstance(value, str) and entry.re.fullmatch(r"\d+\.\d+\.\d+", value), "Exact stable latest version required")
                return tuple(map(int, value.split(".")))
            check(version(latest.get("version")) >= version(row["version"]), "Public latest tag is behind selected stable release")
    return {"manifestHash": entry.chain.digest(manifest), "producerJobId": job["id"], "sourceTree": manifest["sourceTree"],
            "planHash": manifest["planHash"], "packages": actual, "registryInstall": not integrity, "nativeABI": not integrity,
            "latestTargets": True, **({'kind': 'release-byte-integrity-v1', 'registryBytesVerified': True, 'testsExecuted': False} if integrity else {})}


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("stage", choices=["build", "public", "registry"])
    parser.add_argument("--candidate", required=True); parser.add_argument("--output", required=True)
    parser.add_argument("--reports"); args = parser.parse_args()
    transport = entry.actions_transport()
    manifest = json.loads((Path(args.candidate) / "manifest.json").read_text(encoding="utf-8"))
    if args.stage == "registry":
        check(entry.archive_integrity_mode(manifest), "Registry-only receipt requires the signed archive-integrity policy")
        check_archives(manifest, args.candidate, os.environ["GITHUB_SHA"])
        record = registry_record(manifest, current_job(transport, entry.CONSUMER), entry.evidence.registry_module.RegistryReadOnly())
    elif args.stage == "build":
        record = build_record(manifest, entry.evidence.source_reader(os.environ["GITHUB_SHA"]), current_job(transport, entry.PRODUCER), args.candidate)
    else:
        check(args.reports is not None, "Public consumer reports required")
        reports = {platform: json.loads((Path(args.reports) / ("release-public-" + platform) / "result.json").read_text(encoding="utf-8"))
                   for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]}
        record = public_record(manifest, reports, current_job(transport, entry.CONSUMER), entry.evidence.registry_module.RegistryReadOnly())
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"stage": args.stage, "publicationExecuted": False}))


if __name__ == "__main__":
    main()
