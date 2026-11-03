"""Candidate ledger contracts; no publication or build-byte authority is granted."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("ledger_contract", Path(__file__).with_name("ci-execution-ledger.py"))
ledger_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ledger_module)


def fixture():
    env = {"GITHUB_ACTIONS": "true", "GITHUB_JOB": "performance-summary", "GITHUB_REPOSITORY_ID": "1",
           "GITHUB_RUN_ID": "2", "GITHUB_RUN_ATTEMPT": "1", "GITHUB_EVENT_NAME": "pull_request"}
    source = {"sha": "a" * 40, "tree": "b" * 40, "files": {"source.txt": b'product', "docs/experiences/note.md": b'notes'},
              "modes": {"source.txt": "100644", "docs/experiences/note.md": "100644"}, "policyHash": "c" * 64}
    run = {"repositoryId": 1, "runId": 2, "runAttempt": 1, "event": "pull_request"}
    coverage = {"kind": "complete-ci-suite-counts-not-build-bytes", "run": run, "sourceSHA": source["sha"], "sourceTree": source["tree"],
                "cells": [{"kind": kind, "abi": abi, "failed": 0, "skipped": 0, "executedCases": 1,
                           "selectedIds": [kind + ":example"], "executedIds": [kind + ":example"],
                           "caseCounts": [{"id": kind + ":example", "executedCases": 1}]}
                          for kind in ["ts", "examples-browser"] for abi in ledger_module.chain.impact.PRIMARY]}
    return source, coverage, env


class LedgerContracts(unittest.TestCase):
    def test_complete_actual_counts_create_only_a_candidate_e2e_ledger(self):
        source, coverage, env = fixture()
        result = ledger_module.create_ledger(source, coverage, env)
        self.assertEqual(result["scope"], "e2e-only")
        self.assertTrue(result["unitExecutionRequired"])
        self.assertFalse(result["buildByteReuseAuthorized"])
        self.assertFalse(result["publicationAuthorized"])
        self.assertEqual(result["status"], "candidate-requires-independent-completed-job-and-unit-verification")

    def test_missing_skipped_empty_or_wrong_attempt_cannot_create_candidate(self):
        source, coverage, env = fixture()
        for mutate in [lambda data: data["cells"].pop(), lambda data: data["cells"][0].update({"skipped": 1}),
                       lambda data: data["cells"][0].update({"selectedIds": []}), lambda data: data["run"].update({"runAttempt": 2})]:
            bad = copy.deepcopy(coverage); mutate(bad)
            with self.assertRaises(ValueError):
                ledger_module.create_ledger(source, bad, env)
        with self.assertRaises(PermissionError):
            ledger_module.create_ledger(source, coverage, {**env, "GITHUB_JOB": "another-job"})

    def test_document_mode_change_is_not_an_unchanged_product_input(self):
        source, coverage, env = fixture()
        before = ledger_module.create_ledger(source, coverage, env)
        source["modes"]["docs/experiences/note.md"] = "100755"
        after = ledger_module.create_ledger(source, coverage, env)
        self.assertNotEqual(before["sourceSnapshotHash"], after["sourceSnapshotHash"])
        self.assertNotEqual(before["acceptanceInputsHash"], after["acceptanceInputsHash"])


if __name__ == "__main__":
    unittest.main()
