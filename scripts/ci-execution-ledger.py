"""Produce a CI acceptance ledger, never a native byte-reuse or release proof."""
# cspell:words argparse importlib
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location("execution_ledger_runtime", Path(__file__).with_name("incremental-chain-runtime.py"))
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)
chain = runtime.chain


def create_ledger(source, coverage, environment):
    if environment.get("GITHUB_ACTIONS") != "true" or environment.get("GITHUB_JOB") != "performance-summary":
        raise PermissionError("Only the fixed CI baseline producer can record this ledger")
    expected_run = {"repositoryId": int(environment["GITHUB_REPOSITORY_ID"]), "runId": int(environment["GITHUB_RUN_ID"]),
                    "runAttempt": int(environment["GITHUB_RUN_ATTEMPT"]), "event": environment["GITHUB_EVENT_NAME"]}
    if coverage.get("kind") != "complete-ci-suite-counts-not-build-bytes" or coverage.get("run") != expected_run:
        raise ValueError("Actual current-run suite coverage is missing")
    if coverage.get("sourceSHA") != source["sha"] or coverage.get("sourceTree") != source["tree"]:
        raise ValueError("Suite source differs from baseline producer source")
    expected_cells = {kind + ":" + abi for kind in ["ts", "examples-browser"] for abi in chain.impact.PRIMARY}
    rows = coverage.get("cells", [])
    if len(rows) != len(expected_cells) or {row["kind"] + ":" + row["abi"] for row in rows} != expected_cells:
        raise ValueError("Complete three-platform suite execution is required")
    for row in rows:
        if row.get("failed") != 0 or row.get("skipped") != 0 or row.get("executedCases", 0) <= 0:
            raise ValueError("Partial or empty suite execution cannot establish a baseline")
        selected, executed, counts = row.get("selectedIds"), row.get("executedIds"), row.get("caseCounts")
        if (not isinstance(selected, list) or not selected or not isinstance(executed, list) or not isinstance(counts, list)
                or len(selected) != len(set(selected)) or len(executed) != len(set(executed)) or set(selected) != set(executed)
                or len(counts) != len(selected) or {item["id"] for item in counts} != set(selected)
                or any(not isinstance(item.get("executedCases"), int) or isinstance(item["executedCases"], bool) or item["executedCases"] <= 0 for item in counts)
                or sum(item["executedCases"] for item in counts) != row["executedCases"]):
            raise ValueError("Actual semantic discovery and case counts differ")
    records = chain.snapshot_records(source["files"], source["modes"])
    # The initial leaf scope excludes only these private engineering Markdown
    # pages. Every other tracked byte and executable mode remains an input.
    execution_inputs = {file: row for file, row in records.items()
                        if not (file.startswith("docs/experiences/") and file.endswith(".md") and row["mode"] == "100644")}
    result = {"schemaVersion": 2, "kind": "ci-execution-baseline-no-build-byte-authority", "chain": "ci",
              "namespace": "farm-ci-execution-v1", "sourceSHA": source["sha"], "sourceTree": source["tree"],
              "scope": "e2e-only", "unitExecutionRequired": True,
              "sourceSnapshotHash": chain.digest(records), "acceptanceInputsHash": chain.digest(execution_inputs),
              "policyHash": source["policyHash"], "policy": chain.POLICY, "run": expected_run,
              "requiredJobs": runtime.ci_required_jobs(), "producerJob": "Aggregate informational performance report",
              "uploadStep": "Upload trusted CI execution baseline", "coverage": coverage,
              "coverageSha256": hashlib.sha256(json.dumps(coverage, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
              "status": "candidate-requires-independent-completed-job-and-unit-verification",
              "buildByteReuseAuthorized": False, "publicationAuthorized": False}
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--coverage", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    identity = runtime.actions_identity(os.environ)
    source = runtime.evidence.source_reader(identity["sha"])
    coverage_file = Path(args.coverage)
    if coverage_file.stat().st_size > 4 * 1024 * 1024:
        raise ValueError("Suite coverage exceeds bound")
    coverage = json.loads(coverage_file.read_text(encoding="utf-8"))
    ledger = create_ledger(source, coverage, os.environ)
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"kind": ledger["kind"], "cells": len(coverage["cells"]), "buildByteReuseAuthorized": False}))
