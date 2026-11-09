"""Strict reader for historical functional evidence; not a release gate or CLI."""
# cspell:words importlib IFMT ISREG redispatch
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import stat
import zipfile

spec = importlib.util.spec_from_file_location("functional_release_entry", Path(__file__).with_name("incremental-release-entry.py"))
entry = importlib.util.module_from_spec(spec); spec.loader.exec_module(entry)
check = entry.check
retry_spec = importlib.util.spec_from_file_location('release_retry_safety', Path(__file__).with_name('release-retry-safety.py'))
retry = importlib.util.module_from_spec(retry_spec); retry_spec.loader.exec_module(retry)
PLATFORMS = ["ubuntu-24.04", "macos-latest", "windows-latest"]
FUNCTIONAL = ["Release source contracts", "Check declared native inputs (not ABI execution)", "Type Check",
              *[name + " (" + platform + ")" for name in ["Examples Test", "Typescript Test"] for platform in PLATFORMS],
              "Docs / windows-latest", "Docs / ubuntu-24.04", "Record verified CI execution baseline"]
# The other four original functional roles (seal + three candidate consumers)
# are verified from the independent release run by ReleaseVerifier.prepublish.


def exact_plan(body, artifact, member="plan.json"):
    check(0 < len(body) <= 4 * 1024 * 1024 and len(body) == artifact["size_in_bytes"]
          and artifact["digest"] == "sha256:" + hashlib.sha256(body).hexdigest(), "Functional plan archive digest differs")
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        rows = archive.infolist()
        check(len(rows) == 1 and rows[0].filename == member and 0 < rows[0].file_size <= 4 * 1024 * 1024
              and stat.S_IFMT(rows[0].external_attr >> 16) in [0, stat.S_IFREG], "Functional plan archive members differ")
        return json.loads(archive.read(rows[0]).decode("utf-8"))


def main_binding_proof(transport, run, source):
    consumer = entry.load('functional_signed_consumer', 'ci-provenance-consumer.py')
    verifier = entry.load('functional_pinned_cli', 'ci-pinned-attestation-cli.py')
    trust = json.loads(Path(__file__).with_name('ci-provenance-trust.json').read_text())
    check(isinstance(trust, dict) and set(trust) == {'schemaVersion', 'signerSHA'} and trust['schemaVersion'] == 1,
          'Reviewed signer trust configuration differs')
    return consumer.load_verified_run(transport, run, source, trust['signerSHA'], verify=verifier.verify_on_host)


def verify_functional(transport, source, bindings, current_run=None, *, proof_provider=None):
    check(isinstance(bindings, dict) and set(bindings) in [{"ci", "unit", "lint", "spell"}, {"ci", "unit", "lint", "spell", "title"}],
          "Exact CI, Unit and quality workflow acceptance runs required")
    repository = transport.get("")
    accepted_head = None
    accepted_base = None
    for scope, path, planner, required in [
        ("ci", ".github/workflows/ci.yaml", "Incremental CI plan (shadow only)", FUNCTIONAL),
        ("unit", ".github/workflows/rust-test.yaml", "Incremental Unit CI plan (shadow only)", ["Check", *["Test - " + platform for platform in PLATFORMS]])]:
        binding = bindings[scope]
        check(type(binding.get("runId")) is int and binding["runId"] > 0 and type(binding.get("attempt")) is int and binding["attempt"] > 0, "Exact acceptance run/attempt required")
        entry.chain.exact(binding.get("testedSHA"), 40)
        run = transport.get("/actions/runs/" + str(binding["runId"]) + "/attempts/" + str(binding["attempt"]))
        check(run["id"] == binding["runId"] and run["run_attempt"] == binding["attempt"] and run["path"] == path
              and run["status"] == "completed" and run["conclusion"] == "success", "Functional workflow did not succeed")
        check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"]
              and run["event"] in ["pull_request", "workflow_dispatch", "push"], "Functional run repository/event differs")
        latest = transport.get('/actions/runs/' + str(run['id']))
        check(all(latest.get(key) == run.get(key) for key in ['id','run_attempt','path','head_sha','event','status','conclusion']),
              'Functional latest attempt differs; prior success cannot authorize publication')
        tested = transport.get("/git/commits/" + binding["testedSHA"])
        check(tested["tree"]["sha"] == source["tree"], "Actually tested functional tree differs from release source")
        check(run['event'] == 'push' and run['head_branch'] == repository['default_branch']
              and run['head_sha'] == binding['testedSHA'] == source['sha'], 'Functional default-branch checkout differs')
        jobs = entry.execution.jobs_for(transport, run)
        for name in required:
            job = entry.evidence.successful_job(jobs, name)
            check(job.get('run_id') == run['id'] and job.get('head_sha') == run['head_sha']
                  and job.get('run_attempt') == run['run_attempt'], 'Functional current attempt job identity differs')
        expected, signed = (main_binding_proof if proof_provider is None else proof_provider)(transport, run, source)
        check('planner' in signed and signed['planner']['producer'] == planner, 'Signed functional planner role missing')
        consumer = entry.load('functional_attested_binding', 'ci-attested-bindings.py')
        body, artifact, planner_run, planner_job = consumer.bound_artifact(transport, run, signed['planner'], expected)
        plan = exact_plan(body, artifact)
        check(plan.get("testedSHA") == binding["testedSHA"] and plan.get("testedTree") == source["tree"]
              and str(plan.get("runId")) == str(run["id"]) and str(plan.get("runAttempt")) == str(planner_run["run_attempt"])
              and plan.get("repository") == repository["full_name"], "Functional plan execution identity differs")
        check(plan.get("planHash") == entry.chain.digest({key: value for key, value in plan.items() if key != "planHash"}), "Functional plan hash differs")
        check(not plan.get('planningFallback') and not plan.get('unavailable'), 'Functional main source plan unavailable')
        source_plan = importlib.util.spec_from_file_location('functional_source_plan', Path(__file__).with_name('ci-source-plan.py'))
        verifier = importlib.util.module_from_spec(source_plan); source_plan.loader.exec_module(verifier)
        verifier.verify_main_plan(plan, 'e2e' if scope == 'ci' else 'unit')
        if scope == "ci":
            accepted_head, accepted_base = run["head_sha"], plan.get("baseSHA")
    for scope, path in [("lint", ".github/workflows/lint.yaml"), ("spell", ".github/workflows/code-spell-check.yaml"), ("title", ".github/workflows/lint-pr-title.yml")]:
        if scope not in bindings:
            check(scope == 'title' and accepted_head == source['sha'], 'PR title acceptance missing')
            continue
        binding = bindings[scope]
        check(type(binding.get("runId")) is int and binding["runId"] > 0 and type(binding.get("attempt")) is int and binding["attempt"] > 0, "Exact quality workflow identity required")
        run = transport.get("/actions/runs/" + str(binding["runId"]) + "/attempts/" + str(binding["attempt"]))
        own_quality = scope in ['lint', 'spell'] and current_run is not None and current_run['runId'] == run['id'] and 0 < run['run_attempt'] <= current_run['attempt'] and run['path'] == entry.WORKFLOW
        if own_quality:
            check(run['repository']['id'] == repository['id'] == run['head_repository']['id']
                  and run['event'] in ['push', 'workflow_dispatch'] and run['head_branch'] == repository['default_branch']
                  and run['head_sha'] == source['sha'] and run['status'] in ['in_progress', 'completed'], 'Main quality run differs')
            quality = entry.evidence.successful_job(entry.execution.jobs_for(transport, run), 'Release quality checks')
            steps = ['Check release Rust formatting', 'Check release Rust lint', 'Check release TS lint'] if scope == 'lint' else ['Check release spelling']
            for name in steps:
                check(len([step for step in quality.get('steps', []) if step.get('name') == name and step.get('status') == 'completed' and step.get('conclusion') == 'success']) == 1,
                      'Actual main quality step missing or failed')
            continue
        check(run["id"] == binding["runId"] and run["run_attempt"] == binding["attempt"] and run["path"] == path
              and run["repository"]["id"] == repository["id"] == run["head_repository"]["id"]
              and run["status"] == "completed" and run["conclusion"] == "success" and run["head_sha"] == accepted_head,
              "Required quality workflow differs, is stale or did not succeed")
        if run["event"] == "pull_request":
            check(any(row.get("head", {}).get("sha") == accepted_head and row.get("base", {}).get("sha") == accepted_base
                      for row in run.get("pull_requests", [])), "Quality workflow PR merge inputs differ")
        else:
            check(run["event"] in ["push", "workflow_dispatch"] and run["head_branch"] == repository["default_branch"], "Quality workflow event differs")
    return {"sourceTree": source["tree"], "functionalAcceptanceVerified": True, "buildBytesReused": False}
