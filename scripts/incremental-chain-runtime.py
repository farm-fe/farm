"""Actual Actions entrypoint for independently verified chain planning."""
# cspell:words argparse importlib
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess

spec = importlib.util.spec_from_file_location("chain_runtime_evidence", Path(__file__).with_name("incremental-chain-evidence.py"))
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)
chain = evidence.chain_plan


def ci_required_jobs():
    # Normal CI acceptance excludes candidate publication scope. These are
    # exact existing functional jobs, not configurable artifact claims.
    primary = ["Build and Upload User Artifacts - " + abi for abi in chain.impact.PRIMARY]
    extra = ["Build and Upload Release Artifacts - " + abi for abi in chain.impact.TARGETS if abi not in chain.impact.PRIMARY]
    return ["Release source contracts", "Type Check & Changesets", "Check declared native inputs (not ABI execution)",
            *primary, *extra, *[name + " (" + platform + ")" for name in ["Examples Test", "Typescript Test", "Commerce Performance"]
                              for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]],
            "Docs / windows-latest", "Docs / ubuntu-24.04", "Aggregate informational performance report"]


def actions_identity(environment, read_git=chain.impact.git):
    if environment.get("GITHUB_ACTIONS") != "true":
        raise PermissionError("Chain evidence entrypoint runs only in Actions")
    for key in ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_REPOSITORY_ID"]:
        if not re.fullmatch(r"[1-9][0-9]*", environment.get(key, "")):
            raise ValueError("Current Actions identity missing")
    chain.exact(environment.get("GITHUB_SHA"), 40)
    if environment.get("GITHUB_WORKFLOW_SHA") != environment["GITHUB_SHA"]:
        raise ValueError("Workflow and actual source differ")
    if read_git("rev-parse", "HEAD").decode("ascii").strip() != environment["GITHUB_SHA"]:
        raise ValueError("Actual checkout differs from Actions source")
    if environment.get("GITHUB_EVENT_NAME") not in ["pull_request", "push", "workflow_dispatch"]:
        raise ValueError("Unsupported chain event")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", environment.get("GITHUB_REPOSITORY", "")):
        raise ValueError("Exact repository identity required")
    workflow = environment.get("GITHUB_WORKFLOW_REF", "")
    if not workflow.startswith(environment.get("GITHUB_REPOSITORY", "") + "/.github/workflows/ci.yaml@"):
        raise ValueError("Unexpected CI workflow identity")
    return {"sha": environment["GITHUB_SHA"], "tree": read_git("rev-parse", "HEAD^{tree}").decode("ascii").strip()}


def calculate_current_ci(environment=os.environ, transport_factory=evidence.transport_module.GitHubReadOnly,
                         read_source=evidence.source_reader):
    source = actions_identity(environment)
    current = read_source(source["sha"])
    if current["tree"] != source["tree"]:
        raise ValueError("Committed input tree differs from checkout")
    family = json.loads(current["files"]["docs/releases/v2-candidate.json"])
    fallback = chain.plan_chain("ci", source, current["files"], family, [], source_modes=current["modes"])
    reason = None
    try:
        transport = transport_factory(environment["GITHUB_REPOSITORY"], environment["GH_TOKEN"], environment=environment, allow_archive=True)
        repository = transport.get("")
        if repository["id"] != int(environment["GITHUB_REPOSITORY_ID"]):
            raise ValueError("Repository differs from Actions identity")
        branch = transport.get("/branches/" + evidence.transport_module.urllib.parse.quote(repository["default_branch"], safe=""))
        adapter = evidence.BaselineAdapter(transport, repository["id"], branch["commit"]["sha"], "ci", ci_required_jobs(), read_source)
        plan, baseline = evidence.choose_plan(adapter, "ci", source, current["files"], current["modes"], family, [])
    except PermissionError:
        plan, baseline, reason = fallback, None, "evidence-access-denied-full-build"
    except (ValueError, KeyError, OSError, UnicodeError):
        plan, baseline, reason = fallback, None, "evidence-unavailable-full-build"
    plan["currentRun"] = {"repositoryId": int(environment["GITHUB_REPOSITORY_ID"]), "runId": int(environment["GITHUB_RUN_ID"]),
                          "runAttempt": int(environment["GITHUB_RUN_ATTEMPT"]), "event": environment["GITHUB_EVENT_NAME"]}
    plan["baselineBinding"] = baseline["bindingHash"] if baseline else None
    plan["lookupDiagnostic"] = reason or plan.get("lookupDiagnostic")
    # No baseline observation can certify complete build-context measurement.
    # The byte resolver must validate it separately before scheduling omissions.
    plan["executionAuthorized"] = False
    plan.pop("planHash", None)
    plan["planHash"] = chain.digest(plan)
    return plan


def discover_source_suites(source, trusted):
    """Run only the independently approved declarative discovery implementation."""
    if source["policyHash"] != trusted["policyHash"]:
        raise ValueError("Suite discovery source policy is not independently approved")
    module_file = Path(__file__).with_name("ci-suite-inventory.mjs")
    implementation = module_file.read_bytes().replace(b"\r\n", b"\n")
    if implementation != trusted["files"]["scripts/ci-suite-inventory.mjs"].replace(b"\r\n", b"\n"):
        raise ValueError("Loaded suite discovery implementation differs from trusted source")
    chain.snapshot_records(source["files"], source["modes"])
    names = source["files"]
    policy_files = {"vitest.config.ts", "scripts/build.mjs", "scripts/test-examples.mjs", "scripts/test-e2e.mjs",
                    "scripts/test-e2e-worker.mjs", "scripts/check-e2e-specs.mjs", "scripts/test-create-farm-candidates.mjs"}
    selected = {file: body.decode("utf-8") for file, body in names.items()
                if file == "package.json" or file in policy_files or file.endswith(".spec.ts")
                or re.fullmatch(r"(?:packages|js-plugins|examples|rust-plugins)/[^/]+/package.json", file)
                or re.fullmatch(r"examples/[^/]+/(?:index.html|e2e.spec.mjs)", file)
                or file.startswith("e2e/") and re.search(r"\.(?:mjs|cjs|js|ts|json)$", file)}
    directories = sorted({file.split("/")[1] for file in names if file.startswith("examples/") and len(file.split("/")) >= 3})
    data = {"files": selected, "exampleDirectories": directories,
            "sourceIdentity": {"commit": source["sha"], "tree": source["tree"], "trackedInputsOnly": True}}
    body = json.dumps(data, ensure_ascii=True).encode("utf-8")
    if len(body) > 32 * 1024 * 1024:
        raise ValueError("Suite discovery input exceeds bound")
    script = """import fs from 'node:fs';
import {discoverSuiteInventory,makeSuiteExecutionPlan,canonicalSuiteFile,suitePolicy} from './scripts/ci-suite-inventory.mjs';
const data=JSON.parse(fs.readFileSync(0,'utf8')),seen=new Map();
for(const file of [...Object.keys(data.files),...data.exampleDirectories.map(name=>'examples/'+name)]){
canonicalSuiteFile(file);let prefix='';for(const part of file.split('/')){prefix=prefix?prefix+'/'+part:part;
const key=prefix.toLowerCase();if(seen.has(key)&&seen.get(key)!==prefix)throw Error('Suite path collision');seen.set(key,prefix);}}
const inventory=discoverSuiteInventory(data.files,suitePolicy,data.sourceIdentity,data.exampleDirectories);
if(!inventory.fileEnumerationComplete)throw Error('Incomplete committed suite discovery');
const plans=Object.fromEntries(['ts','examples-build','examples-browser','create-farm'].map(kind=>[kind,makeSuiteExecutionPlan(inventory,kind)]));
console.log(JSON.stringify({sourceIdentity:inventory.sourceIdentity,inventoryHash:inventory.inventoryHash,plans}));"""
    result = subprocess.run(["node", "--input-type=module", "-e", script], cwd=module_file.parent.parent,
                            input=body, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=True)
    if len(result.stdout) > 16 * 1024 * 1024:
        raise ValueError("Suite discovery exceeded bound")
    discovered = json.loads(result.stdout.decode("utf-8"))
    if discovered["sourceIdentity"] != data["sourceIdentity"]:
        raise ValueError("Discovered suite source differs")
    return discovered


def discover_current_suites(plan, expected_cells):
    """Derive suites from actual Git inputs and independently read default policy.

    No receipt, uploaded plan, PR input or count document supplies the suite list.
    This is called only by the Actions gate, never during local validation.
    """
    identity = actions_identity(os.environ)
    if identity != {"sha": plan["sourceSHA"], "tree": plan["sourceTree"]}:
        raise ValueError("Suite discovery is not the current execution source")
    transport = evidence.transport_module.GitHubReadOnly(os.environ["GITHUB_REPOSITORY"], os.environ["GH_TOKEN"])
    repository = transport.get("")
    if repository["id"] != plan["currentRun"]["repositoryId"]:
        raise ValueError("Suite policy repository differs")
    route = "/branches/" + evidence.transport_module.urllib.parse.quote(repository["default_branch"], safe="")
    policy_sha = transport.get(route)["commit"]["sha"]
    trusted = evidence.source_reader(policy_sha)
    actual = evidence.source_reader(identity["sha"])
    discovered = discover_source_suites(actual, trusted)
    if transport.get(route)["commit"]["sha"] != policy_sha:
        raise ValueError("Trusted suite policy moved during discovery")
    suites = {}
    for cell in expected_cells:
        category, abi = cell.split(":", 1)
        if abi not in chain.impact.PRIMARY or category not in ["ts-tests", "examples"]:
            raise ValueError("Unsupported semantic execution cell")
        kinds = ["ts"] if category == "ts-tests" else ["examples-build", "examples-browser", "create-farm"]
        ids = sorted(value for kind in kinds for value in discovered["plans"][kind]["ids"])
        if not ids or len(ids) != len(set(ids)):
            raise ValueError("Independent suite inventory is empty or duplicated")
        suites[cell] = {"ids": ids, "inventoryHash": discovered["inventoryHash"]}
    return suites


def validate_execution_results(plan, results, expected_run, expected_cells, *, verified_jobs, semantic_counts,
                               suite_discovery=discover_current_suites):
    payload = {key: value for key, value in plan.items() if key != "planHash"}
    if plan.get("planHash") != chain.digest(payload) or plan.get("currentRun") != expected_run:
        raise ValueError("Execution plan or run identity differs")
    if not expected_cells or len(expected_cells) != len(set(expected_cells)):
        raise ValueError("Trusted execution inventory missing")
    if len(results) != len(expected_cells) or {row.get("cell") for row in results} != set(expected_cells):
        raise ValueError("Missing, duplicate or unexpected execution result")
    expected_suites = suite_discovery(plan, expected_cells)
    if set(expected_suites) != set(expected_cells):
        raise ValueError("Independent semantic policy inventory differs")
    for row in results:
        job = verified_jobs.get(row.get("cell"), {})
        counts = semantic_counts.get(row.get("cell"), {})
        if (job.get("run") != expected_run or job.get("status") != "completed" or job.get("conclusion") != "success"
                or not isinstance(job.get("jobId"), int) or isinstance(job["jobId"], bool) or job["jobId"] <= 0):
            raise ValueError("Independent successful job and attempt evidence missing")
        if (counts.get("producerJobId") != job["jobId"] or counts.get("run") != expected_run
                or counts.get("sourceSHA") != plan["sourceSHA"] or counts.get("sourceTree") != plan["sourceTree"]
                or counts.get("artifactDigestVerified") is not True):
            raise ValueError("Independent runner count artifact provenance missing")
        current_suite = expected_suites[row["cell"]]
        ids = current_suite.get("ids")
        if not isinstance(ids, list) or not ids or len(ids) != len(set(ids)) or any(not isinstance(value, str) or not value for value in ids):
            raise ValueError("Independent semantic suite list is invalid")
        selection = {"cell": row["cell"], "inventoryHash": current_suite["inventoryHash"], "ids": sorted(ids),
                     "sourceSHA": plan["sourceSHA"], "sourceTree": plan["sourceTree"], "run": expected_run,
                     "executionPlanHash": plan["planHash"]}
        if (counts.get("executionPlanHash") != plan["planHash"] or counts.get("suiteInventoryHash") != current_suite["inventoryHash"]
                or counts.get("selectionHash") != chain.digest(selection)):
            raise ValueError("Runner counts are bound to another execution plan or suite selection")
        selected, executed = counts.get("selectedIds"), counts.get("executedIds")
        if (not isinstance(selected, list) or not selected or not isinstance(executed, list)
                or any(not isinstance(value, str) or not value for value in [*selected, *executed])
                or len(selected) != len(set(selected)) or len(executed) != len(set(executed)) or set(selected) != set(executed)
                or set(selected) != set(ids)
                or counts.get("failed") != 0 or counts.get("skipped") != 0):
            raise ValueError("Discovered and actually executed semantic suite inventory differs")
        if row.get("planHash") != plan["planHash"] or row.get("sourceSHA") != plan["sourceSHA"] or row.get("sourceTree") != plan["sourceTree"]:
            raise ValueError("Execution result is bound to another source or plan")
        if row.get("run") != expected_run or row.get("status") != "executed-success":
            raise ValueError("Required execution was skipped, failed or is from another attempt")
        if not isinstance(row.get("executedCount"), int) or isinstance(row["executedCount"], bool) or row["executedCount"] <= 0:
            raise ValueError("Required execution has no actual work count")
        if row["executedCount"] != len(executed):
            raise ValueError("Self-reported execution count differs from verified runner inventory")
    return {"status": "success", "executedCells": len(results), "acceptanceInherited": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args()
    record = calculate_current_ci()
    output = Path(arguments.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"chain": record["chain"], "fullBuild": record["fullBuild"], "buildCount": len(record["build"]),
                      "baseline": record["baselineLookup"] if "baselineLookup" in record else "unavailable",
                      "executionAuthorized": record["executionAuthorized"], "diagnostic": record["lookupDiagnostic"]}))
