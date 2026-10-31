# cspell:words tomllib fnmatch
"""Read committed Git objects only. Planning never authorizes skipping a job."""
import argparse
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import posixpath
import re
import subprocess
import tomllib

SCHEMA = 1
POLICY = "farm-impact-shadow-v1"
ALL = ["native", "rust-tests", "types", "ts-tests", "examples", "docs", "performance", "seal", "consumers", "contracts"]
GLOBAL = {"Cargo.toml", "Cargo.lock", "pnpm-lock.yaml", "pnpm-workspace.yaml", "package.json", "rust-toolchain.toml", "rust-toolchain", "tsconfig.base.json"}
BUILD_POLICY = ["scripts/ci-build-rust-artifacts.sh", "scripts/v2-docker-cargo-cache.mjs", "scripts/check-native-build-node.mjs", "scripts/native-artifact-inventory.mjs"]
TARGETS = ["linux-x64-gnu", "linux-x64-musl", "linux-arm64-gnu", "linux-arm64-musl", "darwin-x64", "darwin-arm64", "win32-x64-msvc", "win32-ia32-msvc", "win32-arm64-msvc"]
PRIMARY = ["linux-x64-gnu", "darwin-arm64", "win32-x64-msvc"]


class WorkspaceIdentityError(ValueError):
    pass


def release_candidate(event, environment):
    pr = event.get("pull_request", {})
    repository = environment.get("GITHUB_REPOSITORY")
    same_repository = bool(repository) and pr.get("head", {}).get("repo", {}).get("full_name") == repository
    return environment.get("GITHUB_EVENT_NAME") == "workflow_dispatch" or (same_repository and (event.get("number") == 2329 or pr.get("head", {}).get("ref", "").startswith("changeset-release/")))


def execution_cells(scope="e2e", release_candidate=True):
    # Full execution remains unchanged. Any workflow matrix/policy change forces
    # full planning and requires updating this reviewed shadow contract.
    if scope == "unit":
        return ["cargo-check", *["rust-tests:ci-test:" + abi for abi in PRIMARY]]
    if scope != "e2e":
        raise ValueError("Unknown execution scope")
    return (["native:ci:" + abi for abi in TARGETS]
            + [suite + ":" + abi for suite in ["examples", "ts-tests", "performance", "performance-contracts"] for abi in PRIMARY]
            + ["docs:release:linux-x64-gnu", "docs:release:win32-x64-msvc", "types", "contracts", "native-assembly", "performance-summary"]
            + (["seal", *["consumers:" + abi for abi in PRIMARY]] if release_candidate else []))


def proposed_dag(suites, examples):
    """Proposed platform-local dependencies only; live needs are not changed."""
    tasks = {"contracts": [], "types": ["contracts"], "cargo-check": ["contracts"]}
    for abi in TARGETS:
        tasks["native:ci:" + abi] = ["contracts"]
    for abi in PRIMARY:
        producer = "native:ci:" + abi
        tasks["ts-tests:" + abi] = [producer]
        tasks["examples:" + abi] = [producer]
        tasks["rust-tests:ci-test:" + abi] = ["contracts"]
        tasks["consumers:" + abi] = ["seal"]
        tasks["performance:" + abi] = ["examples:" + abi, "ts-tests:" + abi, "types", "docs-acceptance"]
    tasks["native-assembly"] = ["native:ci:" + abi for abi in TARGETS]
    tasks["seal"] = ["native-assembly", "types", *["examples:" + abi for abi in PRIMARY], *["ts-tests:" + abi for abi in PRIMARY]]
    tasks["docs:release:linux-x64-gnu"] = ["contracts"]
    tasks["docs:release:win32-x64-msvc"] = ["contracts"]
    tasks["docs-acceptance"] = ["docs:release:linux-x64-gnu", "docs:release:win32-x64-msvc"]
    return {"tasks": tasks, "selectedSuites": suites, "exampleDirectories": examples,
            "docsSuites": ["source", "distribution", "platform", "compiled", "browser", "diagrams", "notices-source", "notices-built", "archive-python"] if "docs" in suites else [],
            "executionEnabled": False, "note": "Semantic suites and platform-local needs are a shadow proposal, not task omission authority."}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()


def git(*args):
    result = subprocess.run(["git", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=True)
    return result.stdout


def revision(value, read_git=None):
    if not re.fullmatch(r"[a-f0-9]{40}", value or ""):
        raise ValueError("Exact forty-character Git identity required")
    read_git = read_git or git
    if read_git("rev-parse", value + "^{commit}").decode("ascii").strip() != value:
        raise ValueError("Commit identity differs")
    return value


def tree_revision(value, read_git=None):
    if not re.fullmatch(r"[a-f0-9]{40}", value or ""):
        raise ValueError("Exact forty-character Git tree identity required")
    read_git = read_git or git
    if read_git("cat-file", "-t", value).decode("ascii").strip() != "tree":
        raise ValueError("Tree object type differs")
    return value


def snapshot(commit):
    records = []
    total = 0
    for entry in git("ls-tree", "-rzl", commit).split(b"\0"):
        if not entry:
            continue
        metadata, name = entry.split(b"\t", 1)
        mode, kind, oid, size = metadata.decode().split()
        if kind != "blob" or mode != "100644" and mode != "100755":
            raise ValueError("Tracked symlink or submodule requires a full run")
        records.append((name.decode("utf-8"), mode, oid))
        total += int(size)
        if total > 256 * 1024 * 1024:
            raise ValueError("Bounded source bytes exceeded")
    if len(records) > 30000:
        raise ValueError("Bounded source snapshot exceeded")
    # One bounded process avoids thousands of per-file Git subprocesses.
    data = subprocess.run(["git", "cat-file", "--batch"], input="".join(oid + "\n" for _, _, oid in records).encode(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90, check=True).stdout
    if len(data) > 256 * 1024 * 1024:
        raise ValueError("Bounded source bytes exceeded")
    files, hashes, cursor = {}, {}, 0
    for name, mode, oid in records:
        end = data.index(b"\n", cursor)
        header = data[cursor:end].decode().split()
        if header[:2] != [oid, "blob"]:
            raise ValueError("Git blob identity differs")
        size = int(header[2]); start = end + 1
        body = data[start:start + size]
        if len(body) != size or data[start + size:start + size + 1] != b"\n":
            raise ValueError("Incomplete Git blob")
        files[name] = body
        hashes[name] = {"mode": mode, "sha256": hashlib.sha256(body).hexdigest()}
        cursor = start + size + 1
    if cursor != len(data):
        raise ValueError("Unexpected Git batch suffix")
    return files, hashes


def matches(directory, pattern):
    return directory.count("/") == pattern.count("/") and fnmatch.fnmatchcase(directory, pattern)


def graph(files):
    workspace = tomllib.loads(files["Cargo.toml"].decode())
    cargo_patterns = workspace["workspace"]["members"]
    workspace_dependencies = workspace.get("workspace", {}).get("dependencies", {})
    # The current workspace list has a deliberately small, explicit YAML shape.
    lines = files["pnpm-workspace.yaml"].decode().splitlines()
    if any(line.strip() and line.strip() != "packages:" and not re.fullmatch(r"\s*-\s+[\w*/.-]+\s*", line) and not line.lstrip().startswith("#") for line in lines):
        raise ValueError("Workspace YAML shape requires policy review")
    js_patterns = [line.strip()[2:].strip() for line in lines if line.strip().startswith("- ")]
    nodes, by_name = {}, {}
    for file, body in files.items():
        directory = posixpath.dirname(file)
        cargo = file.endswith("/Cargo.toml") and (any(matches(directory, pattern) for pattern in cargo_patterns) or directory.startswith("patches/"))
        js = file.endswith("/package.json") and any(matches(directory, pattern) for pattern in js_patterns)
        if not cargo and not js:
            continue
        manifest = tomllib.loads(body.decode()) if cargo else json.loads(body)
        package = manifest.get("package", {}) if cargo else manifest
        if not package.get("name"):
            continue
        key = ("cargo:" if cargo else "npm:") + package["name"]
        if key in nodes:
            raise WorkspaceIdentityError("Duplicate workspace package identity")
        nodes[key] = {"directory": directory, "manifest": file, "kind": "cargo" if cargo else "npm", "data": manifest, "productDeps": set(), "testDeps": set()}
        by_name[key] = key
    by_directory = {node["directory"]: key for key, node in nodes.items() if node["kind"] == "cargo"}
    for key, node in nodes.items():
        manifest = node["data"]
        if node["kind"] == "cargo":
            tables = [manifest, *manifest.get("target", {}).values()]
            for table in tables:
                for kind in ["dependencies", "build-dependencies", "dev-dependencies"]:
                    for alias, declaration in table.get(kind, {}).items():
                        dep = declaration if isinstance(declaration, dict) else {}
                        inherited = bool(dep.get("workspace"))
                        if inherited:
                            declared = workspace_dependencies.get(alias, {})
                            dep = {**(declared if isinstance(declared, dict) else {}), **dep}
                        name = "cargo:" + dep.get("package", alias)
                        target = None
                        if "path" in dep:
                            target = by_directory.get(posixpath.normpath(posixpath.join("" if inherited else node["directory"], dep["path"])))
                            if not target:
                                raise ValueError("Unresolved repository Cargo path dependency")
                        elif name in nodes:
                            target = name
                        if target:
                            node["testDeps"].add(target)
                            if kind != "dev-dependencies":
                                node["productDeps"].add(target)
        else:
            for kind in ["dependencies", "optionalDependencies", "devDependencies", "peerDependencies"]:
                for name in manifest.get(kind, {}):
                    target = "npm:" + name
                    if target in nodes:
                        # Build configs import workspace dev dependencies too.
                        node["productDeps"].add(target); node["testDeps"].add(target)
    # Native owners bridge the JS and Cargo graphs; package tools build them.
    for key, node in nodes.items():
        if node["kind"] == "npm":
            directory = node["directory"]
            # Core is the explicit JS wrapper for crates/node. Other dual
            # packages bind the Cargo manifest actually present in that directory.
            cargo_directory = "crates/node" if directory == "packages/core" else directory
            if cargo_directory in by_directory:
                node["productDeps"].add(by_directory[cargo_directory]); node["testDeps"].add(by_directory[cargo_directory])
            if directory.startswith("rust-plugins/"):
                node["productDeps"].add("npm:@farmfe/plugin-tools"); node["testDeps"].add("npm:@farmfe/plugin-tools")
    return nodes


def reverse_closure(nodes, selected, edge):
    result = set(selected)
    while True:
        expanded = result | {name for name, node in nodes.items() if node[edge] & result}
        if expanded == result:
            return sorted(result)
        result = expanded


def owners(nodes, file):
    candidates = [(len(node["directory"]), name) for name, node in nodes.items() if file.startswith(node["directory"] + "/")]
    if not candidates:
        return set()
    depth = max(length for length, _ in candidates)
    return {name for length, name in candidates if length == depth}


def test_only(file):
    return "/tests/" in file or "/__tests__/" in file or "/__snapshots__/" in file or file.endswith((".test.mjs", ".test.cjs", ".spec.ts", ".spec.mjs"))


def plan_changes(base_files, head_files, changes, extra_snapshots=(), proven_test_inputs=frozenset()):
    base, head = graph(base_files), graph(head_files)
    nodes = {}
    graphs = [base, head, *[graph(files) for files in extra_snapshots]]
    for current in graphs:
        for name, node in current.items():
            prior = nodes.get(name, {})
            nodes[name] = {**prior, **node, "productDeps": prior.get("productDeps", set()) | node["productDeps"], "testDeps": prior.get("testDeps", set()) | node["testDeps"]}
    products, tests, suites, reasons = set(), set(), {"contracts"}, []
    full = False
    for change in changes:
        for file in change["paths"]:
            if file in GLOBAL or file.startswith((".github/", ".cargo/", "patches/")) or file in BUILD_POLICY or file.startswith("scripts/ci-impact") or file.startswith("scripts/ci-native-proof"):
                full = True; reasons.append({"file": file, "reason": "global-or-policy-input"}); continue
            names = set().union(*(owners(current, file) for current in graphs))
            if names:
                proven = file in proven_test_inputs
                (tests if proven else products).update(names)
                reasons.append({"file": file, "owners": sorted(names), "testOnlyCandidate": test_only(file), "testOnlyProven": proven})
            elif file.startswith(("scripts/verify-v2-", "scripts/v2-consumer-", "scripts/verify-published-")):
                suites.update(["consumers", "seal"])
            elif file.startswith(("scripts/performance", "scripts/aggregate-performance", "scripts/prepare-performance", "scripts/business-")):
                suites.add("performance")
            elif file.startswith(("scripts/seal-", "scripts/sdk-", "scripts/prepare-sdk-", "scripts/pack-v2-", "scripts/v2-rust-", "scripts/release", "docs/releases/", "docs/release-evidence/")):
                suites.update(["seal", "consumers"])
            elif file.startswith(("scripts/farm-docs-", "docs/farm-docs/")):
                suites.add("docs")
            else:
                full = True; reasons.append({"file": file, "reason": "unclassified-conservative-full"})
    product_closure = reverse_closure(nodes, products, "productDeps")
    test_closure = reverse_closure(nodes, products | tests, "testDeps")
    for name in product_closure + test_closure:
        node = nodes[name]; directory = node["directory"]
        if node["kind"] == "cargo":
            suites.add("rust-tests")
        else:
            suites.update(["types", "ts-tests"])
        if directory.startswith("examples/"):
            suites.add("examples")
        if "docs" in directory or directory == "website":
            suites.add("docs")
        if directory in ["packages/core", "packages/cli", "packages/runtime", "packages/runtime-plugin-hmr"]:
            suites.update(["examples", "docs", "performance"])
    if any(nodes[name]["kind"] == "cargo" for name in product_closure):
        suites.add("native")
    if products:
        suites.update(["seal", "consumers"])
    return {"fullRequired": full, "reasons": reasons, "productNodes": product_closure, "testNodes": test_closure,
            "proposedSuites": ALL if full else sorted(suites), "proposedExamples": sorted(nodes[name]["directory"] for name in test_closure if nodes[name]["directory"].startswith("examples/")),
            "graph": {name: {key: sorted(value) if isinstance(value, set) else value for key, value in node.items() if key != "data"} for name, node in sorted(nodes.items())}}


def fingerprints(nodes, hashes, proven_test_inputs=frozenset()):
    result = {}
    for name, node in nodes.items():
        closure, pending = set(), [name]
        while pending:
            item = pending.pop()
            if item in closure:
                continue
            closure.add(item); pending.extend(nodes[item]["productDeps"])
        prefixes = [nodes[item]["directory"] + "/" for item in closure]
        product = {file: value for file, value in hashes.items() if (any(file.startswith(prefix) for prefix in prefixes) and file not in proven_test_inputs) or file in GLOBAL or file in BUILD_POLICY or file.startswith((".cargo/", "patches/"))}
        local_tests = {file: value for file, value in hashes.items() if file.startswith(node["directory"] + "/") and test_only(file)}
        test_closure, pending = set(), [name]
        while pending:
            item = pending.pop()
            if item in test_closure:
                continue
            test_closure.add(item); pending.extend(nodes[item]["testDeps"])
        test_prefixes = [nodes[item]["directory"] + "/" for item in test_closure]
        acceptance = {file: value for file, value in hashes.items() if any(file.startswith(prefix) for prefix in test_prefixes) or file in GLOBAL or file.startswith(("scripts/", ".github/", ".cargo/", "patches/"))}
        result[name] = {"product": digest(product), "localTestSource": digest(local_tests), "tests": digest(acceptance), "closure": sorted(closure), "testClosure": sorted(test_closure), "acceptanceKeyComplete": False}
    return result


def changed_paths(base, head):
    parts = git("diff", "--name-status", "-z", "--find-renames", base, head).split(b"\0")
    changes, index = [], 0
    while index < len(parts) and parts[index]:
        status = parts[index].decode(); index += 1
        count = 2 if status.startswith(("R", "C")) else 1
        paths = [item.decode("utf-8") for item in parts[index:index + count]]; index += count
        if len(paths) != count or any(not item or item.startswith("/") or ".." in item.split("/") for item in paths):
            raise ValueError("Invalid changed path")
        changes.append({"status": status, "paths": paths})
    return changes


def make_plan(base, head, tested, scope="e2e", release_candidate=True):
    revision(base); revision(head); revision(tested)
    bases = git("merge-base", "--all", base, head).decode().split()
    if len(bases) != 1:
        raise ValueError("Ambiguous merge base")
    snapshots = {}
    def read_snapshot(commit):
        if commit not in snapshots:
            snapshots[commit] = snapshot(commit)
        return snapshots[commit]
    base_files, base_hashes = read_snapshot(bases[0]); head_files, head_hashes = read_snapshot(head)
    base_tip_files, base_tip_hashes = read_snapshot(base); tested_files, tested_hashes = read_snapshot(tested)
    changes = changed_paths(bases[0], head)
    tested_changes = changed_paths(head, tested)
    plan = plan_changes(base_files, head_files, changes + tested_changes, extra_snapshots=[base_tip_files, tested_files])
    plan["proposedDag"] = proposed_dag(plan["proposedSuites"], plan["proposedExamples"])
    plan.update({"schemaVersion": SCHEMA, "policy": POLICY, "mode": "shadow", "skipAuthorized": False,
                 "baseSHA": base, "mergeBaseSHA": bases[0], "headSHA": head, "testedSHA": tested,
                 "testedTree": git("rev-parse", tested + "^{tree}").decode().strip(), "changes": changes, "testedMergeChanges": tested_changes,
                 "fingerprints": {"mergeBase": fingerprints(graph(base_files), base_hashes), "base": fingerprints(graph(base_tip_files), base_tip_hashes), "head": fingerprints(graph(head_files), head_hashes), "tested": fingerprints(graph(tested_files), tested_hashes)},
                 "executedSuites": ALL, "reuseAuthorized": False, "profiles": ["ci", "ci-test", "release"],
                 "runId": os.environ.get("GITHUB_RUN_ID"), "runAttempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
                 "repository": os.environ.get("GITHUB_REPOSITORY"), "producerTrust": "not-enabled", "executionScope": scope, "executionCells": execution_cells(scope, release_candidate),
                 "baseTree": git("rev-parse", base + "^{tree}").decode().strip(), "headTree": git("rev-parse", head + "^{tree}").decode().strip()})
    plan["planHash"] = digest(plan)
    return plan


def conservative_plan(base, head, tested, scope, candidate, error, environment, read_git=None):
    """Full execution fallback retains only independently verified identity."""
    read_git = read_git or git
    revision(base, read_git); revision(head, read_git); revision(tested, read_git)
    if read_git("rev-parse", "HEAD").decode("ascii").strip() != tested:
        raise ValueError("Fallback checkout differs from tested commit")
    tree = tree_revision(read_git("rev-parse", "HEAD^{tree}").decode("ascii").strip(), read_git)
    if read_git("rev-parse", tested + "^{tree}").decode("ascii").strip() != tree:
        raise ValueError("Fallback tree differs from tested commit tree")
    for key in ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT"]:
        if not re.fullmatch(r"[1-9][0-9]*", environment.get(key, "")):
            raise ValueError("Fallback run identity unavailable")
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", environment.get("GITHUB_REPOSITORY", "")):
        raise ValueError("Fallback repository identity unavailable")
    classes = ["WorkspaceIdentityError", "ValueError", "KeyError", "TimeoutExpired", "CalledProcessError", "UnicodeDecodeError", "JSONDecodeError", "TOMLDecodeError"]
    result = {"schemaVersion": SCHEMA, "policy": POLICY, "mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False,
              "fullRequired": True, "planningFallback": True, "sourceInputProofComplete": False,
              "reasons": [{"reason": "planning-failed", "errorClass": type(error).__name__ if type(error).__name__ in classes else "unknown"}],
              "baseSHA": base, "headSHA": head, "testedSHA": tested, "testedTree": tree,
              "runId": environment["GITHUB_RUN_ID"], "runAttempt": environment["GITHUB_RUN_ATTEMPT"], "repository": environment["GITHUB_REPOSITORY"],
              "executionScope": scope, "executionCells": execution_cells(scope, candidate), "proposedSuites": ALL, "executedSuites": ALL, "producerTrust": "not-enabled"}
    result["planHash"] = digest(result)
    return result


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--output", required=True); parser.add_argument("--event", default=os.environ.get("GITHUB_EVENT_PATH")); args = parser.parse_args()
    base = head = None; candidate = False
    try:
        event = json.loads(Path(args.event).read_text(encoding="utf-8"))
        pr = event.get("pull_request", {})
        base = pr.get("base", {}).get("sha") or event.get("inputs", {}).get("release_base")
        head = pr.get("head", {}).get("sha") or os.environ.get("GITHUB_SHA")
        candidate = release_candidate(event, os.environ)
        result = make_plan(base, head, os.environ.get("GITHUB_SHA"), os.environ.get("CI_IMPACT_SCOPE", "e2e"), candidate)
    except Exception as error:
        try:
            result = conservative_plan(base, head, os.environ.get("GITHUB_SHA"), os.environ.get("CI_IMPACT_SCOPE", "e2e"), candidate, error, os.environ)
        except Exception:
            result = {"schemaVersion": SCHEMA, "policy": POLICY, "mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False,
                      "fullRequired": True, "unavailable": "fallback identity could not be verified", "reasons": [{"reason": "planning-failed"}], "proposedSuites": ALL, "executedSuites": ALL}
            result["planHash"] = digest(result)
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True); output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: result[key] for key in ["mode", "planHash", "fullRequired", "skipAuthorized", "proposedSuites"]}))


if __name__ == "__main__":
    main()
