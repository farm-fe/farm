"""Independent source discovery must reject mutually consistent incomplete receipts."""
# cspell:words importlib urllib
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import urllib.parse
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    result = importlib.util.module_from_spec(spec); spec.loader.exec_module(result)
    return result


proof = load("suite_source_proof", "ci-execution-evidence.py")
producer = load("suite_source_producer", "ci-execution-ledger.py")


def fixture():
    run_identity = {"repositoryId": 1, "runId": 2, "runAttempt": 1, "event": "push"}
    source = {"sha": "a" * 40, "tree": "b" * 40, "parents": [], "files": {"source.txt": b'product'},
              "modes": {"source.txt": "100644"}, "policyHash": "c" * 64}
    discovered = {"sourceIdentity": {"commit": source["sha"], "tree": source["tree"]}, "inventoryHash": "d" * 64,
                  "plans": {kind: {"ids": [kind + ":first", kind + ":last"], "planHash": value * 64}
                            for kind, value in [("ts", "e"), ("examples-browser", "f")]}}
    cells = []
    for abi in proof.PLATFORMS:
        for kind in ["ts", "examples-browser"]:
            expected = discovered["plans"][kind]; label = "ts" if kind == "ts" else "browser"
            cells.append({"kind": kind, "abi": abi, "sourceSHA": source["sha"], "sourceTree": source["tree"], "run": run_identity,
                          "selectedIds": expected["ids"].copy(), "executedIds": expected["ids"].copy(), "executedCases": 2,
                          "caseCounts": [{"id": item, "executedCases": 1} for item in expected["ids"]], "failed": 0, "skipped": 0,
                          "inventoryHash": discovered["inventoryHash"], "planHash": expected["planHash"],
                          "artifactName": "farm-ci-suite-" + label + "-" + source["sha"] + "-" + abi + "-1"})
    coverage = {"kind": "complete-ci-suite-counts-not-build-bytes", "run": run_identity, "sourceSHA": source["sha"], "sourceTree": source["tree"], "cells": cells}
    env = {"GITHUB_ACTIONS": "true", "GITHUB_JOB": "performance-summary", "GITHUB_REPOSITORY_ID": "1", "GITHUB_RUN_ID": "2",
           "GITHUB_RUN_ATTEMPT": "1", "GITHUB_EVENT_NAME": "push"}
    ledger = producer.create_ledger(source, coverage, env)
    run = {"id": 2, "run_attempt": 1, "repository": {"id": 1}, "head_repository": {"id": 1}, "path": ".github/workflows/ci.yaml",
           "status": "completed", "conclusion": "success", "event": "push", "head_branch": "main", "head_sha": source["sha"]}
    repository = {"id": 1, "default_branch": "main"}
    def step(name):
        return {"name": name, "status": "completed", "conclusion": "success", "started_at": "2026-10-31T01:01:00Z", "completed_at": "2026-10-31T01:03:00Z"}
    jobs = [{"id": index + 10, "run_id": 2, "name": name, "status": "completed", "conclusion": "success", "started_at":"2026-10-31T01:00:00Z", "completed_at":"2026-10-31T01:04:00Z", "steps": []}
            for index, name in enumerate(ledger["requiredJobs"])]
    for job in jobs:
        if job["name"] == proof.PRODUCER:
            job["steps"] = [step(proof.UPLOAD)]
        elif job["name"].startswith("Examples Test ("):
            job["steps"] = [step("Preserve actual browser discovery and execution counts")]
        elif job["name"].startswith("Typescript Test ("):
            job["steps"] = [step("Preserve actual TypeScript discovery and execution counts")]
    artifact = {"id": 3, "name": "farm-ci-execution-baseline-2-1", "workflow_run": {"id": 2}, "expired": False,
                "created_at": "2026-10-31T01:02:00Z"}
    class Transport:
        def __init__(self):
            self.rows, self.bytes = {}, {}
        def pack(self):
            self.rows, self.bytes = {}, {}
            ledger_zip = io.BytesIO()
            with zipfile.ZipFile(ledger_zip, "w") as archive:
                archive.writestr("ledger.json", json.dumps(ledger))
            ledger_bytes = ledger_zip.getvalue()
            artifact.update({"size_in_bytes": len(ledger_bytes), "digest": "sha256:" + hashlib.sha256(ledger_bytes).hexdigest()})
            self.bytes[artifact["id"]] = ledger_bytes
            for index, cell in enumerate(ledger["coverage"]["cells"]):
                identity = index + 100; name = cell["artifactName"]
                body = io.BytesIO()
                with zipfile.ZipFile(body, "w") as archive:
                    archive.writestr("ts.json" if cell["kind"] == "ts" else "browser.json", json.dumps({key: value for key, value in cell.items() if key != "artifactName"}))
                data = body.getvalue()
                self.bytes[identity] = data
                self.rows[name] = {**artifact, "id": identity, "name": name, "size_in_bytes": len(data), "digest": "sha256:" + hashlib.sha256(data).hexdigest()}
        def get(self, route):
            if "/artifacts?" in route:
                name = urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)["name"][0]
                if name == artifact["name"]:
                    return {"total_count": 1, "artifacts": [artifact]}
                return {"total_count": 1 if name in self.rows else 0, "artifacts": [self.rows[name]] if name in self.rows else []}
            identity = int(route.rsplit("/", 1)[1])
            if identity == artifact["id"]:
                return artifact
            return next(row for row in self.rows.values() if row["id"] == identity)
        def archive(self, identity):
            return self.bytes[identity]
    transport = Transport(); transport.pack()
    args = [transport, run, repository, ledger, artifact, jobs, source, source, ledger["requiredJobs"]]
    return args, discovered


def verify(args, discovery):
    return proof.verify_execution_ledger(*args, suite_discovery=lambda *_: copy.deepcopy(discovery))


class SourceSuiteContracts(unittest.TestCase):
    def test_browser_exclusions_require_independent_source_policy(self):
        args, discovery = fixture()
        declared = [{'id': 'examples-browser:first', 'name': 'run preview', 'sourceDigest': 'a' * 64, 'reason': 'Reviewed source exclusion'}]
        discovery['plans']['examples-browser']['excludedCases'] = declared
        for cell in args[3]['coverage']['cells']:
            if cell['kind'] == 'examples-browser': cell['excludedCases'] = copy.deepcopy(declared)
        args[0].pack()
        self.assertFalse(verify(args, discovery)['buildBytesReused'])
        for cell in args[3]['coverage']['cells']:
            if cell['kind'] == 'examples-browser': cell['excludedCases'][0]['name'] = 'unexpected omission'
        args[0].pack()
        with self.assertRaisesRegex(ValueError, 'browser exclusions differ'):
            verify(args, discovery)

    def test_real_zip_receipts_match_independent_complete_source_inventory(self):
        args, discovery = fixture()
        original, metadata = proof.fetch_json_artifact(args[0], args[1], args[4]["name"])
        self.assertEqual(original, args[3])
        self.assertEqual(metadata["digest"], args[4]["digest"])
        self.assertFalse(verify(args, discovery)["buildBytesReused"])

    def test_consistent_missing_extra_suite_or_forged_plan_hash_is_rejected(self):
        for kind in ["missing", "extra", "hash"]:
            args, discovery = fixture(); cell = args[3]["coverage"]["cells"][0]
            if kind == "hash":
                cell["planHash"] = "0" * 64
            elif kind == "missing":
                cell["selectedIds"] = cell["selectedIds"][:1]; cell["executedIds"] = cell["executedIds"][:1]
                cell["caseCounts"] = cell["caseCounts"][:1]; cell["executedCases"] = 1
            else:
                cell["selectedIds"].append("ts:extra"); cell["executedIds"].append("ts:extra")
                cell["caseCounts"].append({"id": "ts:extra", "executedCases": 1}); cell["executedCases"] = 3
            args[0].pack()  # Both the ledger and original ZIP now agree on the bad subset.
            with self.assertRaisesRegex(ValueError, "independent source discovery"):
                verify(args, discovery)

    def test_corrupt_zip_digest_missing_receipt_failed_job_and_upload_are_rejected(self):
        for kind in ["digest", "missing", "job", "upload"]:
            args, discovery = fixture()
            if kind == "digest":
                next(iter(args[0].rows.values()))["digest"] = "sha256:" + "0" * 64
            elif kind == "missing":
                args[0].rows.pop(next(iter(args[0].rows)))
            elif kind == "job":
                args[5][0]["conclusion"] = "failure"
            else:
                next(job for job in args[5] if job["name"] == proof.PRODUCER)["steps"] = []
            with self.assertRaises(ValueError):
                verify(args, discovery)


if __name__ == "__main__":
    unittest.main()
