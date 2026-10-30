# cspell:words importlib
"""Current-cell binding fixtures, never real builds or GitHub requests."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("receipt", Path(__file__).with_name("ci-impact-receipt.py"))
receipt = importlib.util.module_from_spec(spec); spec.loader.exec_module(receipt)


class ReceiptContracts(unittest.TestCase):
    def test_observation_before_plan_is_strictly_bound_after_plan_completes(self):
        plan, env = self.fixture()
        env = {**env, "GITHUB_SHA": "c" * 40, "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_WORKFLOW_REF": "farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2329/merge"}
        event = {"pull_request": {"head": {"sha": "a" * 40}, "base": {"sha": "b" * 40}}}
        for status in ["success", "failure", "cancelled"]:
            observed = receipt.make_observation("examples:linux-x64-gnu", status, env, event, "c" * 40, "d" * 40)
            self.assertNotIn("planHash", observed)
            self.assertFalse(observed["planBound"])
            bound = receipt.bind_observation(plan, observed)
            self.assertEqual(bound["planHash"], plan["planHash"])
            self.assertEqual(bound["status"], "executed-" + status)
            self.assertTrue(bound["planBound"])
            self.assertFalse(bound["authoritativeJobIdentityVerified"])
        for key, value in [("runAttempt", "1"), ("testedTree", "e" * 40), ("baseSHA", "e" * 40), ("cell", "unknown"), ("workflowRef", None)]:
            changed = {**observed, key: value}
            changed["observationHash"] = receipt.impact.digest({k: v for k, v in changed.items() if k != "observationHash"})
            with self.assertRaises(ValueError):
                receipt.bind_observation(plan, changed)
        with self.assertRaises(ValueError):
            receipt.bind_observation({}, observed)
        with self.assertRaises(ValueError):
            receipt.bind_observation(plan, {**observed, "status": "executed-success"})

    def test_cell_action_records_without_downloading_or_waiting_for_plan(self):
        action = (Path(__file__).parents[1] / ".github/actions/ci-impact-cell/action.yml").read_text()
        self.assertNotIn("download-artifact", action)
        self.assertNotIn("--plan", action)
        self.assertIn("CI_IMPACT_JOB_STATUS: ${{ inputs.status }}", action)
        self.assertIn("python scripts/ci-impact-receipt.py --output", action)

    def fixture(self):
        plan = {"mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False, "headSHA": "a" * 40, "baseSHA": "b" * 40,
                "testedSHA": "c" * 40, "testedTree": "d" * 40, "executionCells": ["examples:linux-x64-gnu"], "runId": "10", "runAttempt": "2", "policy": receipt.impact.POLICY, "repository": "farm-fe/farm"}
        plan["planHash"] = receipt.impact.digest(plan)
        env = {"GITHUB_RUN_ID": "10", "GITHUB_RUN_ATTEMPT": "2", "GITHUB_JOB": "examples-test"}
        return plan, env

    def test_real_outcome_is_preserved_without_granting_authority(self):
        plan, env = self.fixture()
        for status in ["success", "failure", "cancelled"]:
            value = receipt.make_receipt(plan, "examples:linux-x64-gnu", status, env, "c" * 40, "d" * 40)
            self.assertEqual(value["status"], "executed-" + status)
            self.assertFalse(value["acceptanceInherited"]); self.assertFalse(value["authoritativeJobIdentityVerified"])

    def test_old_checkout_attempt_or_unselected_cell_is_rejected(self):
        plan, env = self.fixture()
        for cell, environment, sha, tree in [("native:ci:linux-x64-gnu", env, "c" * 40, "d" * 40),
                                              ("examples:linux-x64-gnu", {**env, "GITHUB_RUN_ATTEMPT": "1"}, "c" * 40, "d" * 40),
                                              ("examples:linux-x64-gnu", env, "e" * 40, "d" * 40),
                                              ("examples:linux-x64-gnu", env, "c" * 40, "e" * 40)]:
            with self.assertRaises(ValueError):
                receipt.make_receipt(plan, cell, "success", environment, sha, tree)

    def test_unreviewed_enable_flag_and_unknown_status_are_rejected(self):
        plan, env = self.fixture()
        with self.assertRaises(ValueError):
            receipt.make_receipt(plan, "examples:linux-x64-gnu", "skipped", env, "c" * 40, "d" * 40)
        plan["skipAuthorized"] = True
        plan["planHash"] = receipt.impact.digest({key: value for key, value in plan.items() if key != "planHash"})
        with self.assertRaises(ValueError):
            receipt.make_receipt(plan, "examples:linux-x64-gnu", "success", env, "c" * 40, "d" * 40)


if __name__ == "__main__":
    unittest.main()
