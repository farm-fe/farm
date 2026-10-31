# cspell:words importlib charmap contextlib
"""Current-cell binding fixtures, never real builds or GitHub requests."""
import importlib.util
import contextlib
import io
import json
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("receipt", Path(__file__).with_name("ci-impact-receipt.py"))
receipt = importlib.util.module_from_spec(spec); spec.loader.exec_module(receipt)
summary_spec = importlib.util.spec_from_file_location("summary", Path(__file__).with_name("ci-impact-summary.py"))
summary = importlib.util.module_from_spec(summary_spec); summary_spec.loader.exec_module(summary)


class ReceiptContracts(unittest.TestCase):
    def test_binding_diagnostic_is_a_finite_reason_and_field_without_values(self):
        plan, env = self.fixture()
        environment = {**env, "GITHUB_SHA": "c" * 40, "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_WORKFLOW_REF": "farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge"}
        event = {"pull_request": {"head": {"sha": "a" * 40}, "base": {"sha": "b" * 40}}}
        observed = receipt.make_observation("examples:linux-x64-gnu", "success", environment, event, "c" * 40, "d" * 40)
        changed = {**observed, "testedTree": "e" * 40}
        changed["observationHash"] = receipt.impact.digest({key: value for key, value in changed.items() if key != "observationHash"})
        with self.assertRaises(receipt.BindingError) as failure:
            receipt.bind_observation(plan, changed)
        failure.exception.cell = observed["cell"]
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            detail = receipt.report_error("bind-observations", failure.exception)
            injected = receipt.report_error("bind-observations", receipt.BindingError("secret-token", "secret-token"))
        self.assertEqual(detail["code"], "identity-mismatch")
        self.assertEqual(detail["identityField"], "testedTree")
        self.assertEqual(detail["cell"], observed["cell"])
        self.assertEqual(injected["code"], "validation-failed")
        self.assertNotIn("secret-token", stderr.getvalue())
        self.assertNotIn("e" * 40, stderr.getvalue())

    def test_untrusted_diagnostic_tokens_are_replaced_with_unknown(self):
        for field in ["stage", "class", "code", "encoding"]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temporary:
                error = {"stage": "read-event", "class": "UnicodeDecodeError", "code": "invalid-text-encoding", "encoding": "utf-8", field: "secret-token"}
                source = Path(temporary) / "receipt.json"
                source.write_text(json.dumps({"cell": "cargo-check", "status": "unavailable", "error": error}), encoding="utf-8")
                _, unavailable = summary.read_observations(temporary)
                self.assertEqual(unavailable[0]["error"][field], "unknown")
                self.assertNotIn("secret-token", json.dumps(unavailable))

    def test_main_reads_utf8_event_under_a_legacy_default_encoding(self):
        _, env = self.fixture()
        event = {"title": "中文 🚀", "pull_request": {"head": {"sha": "a" * 40}, "base": {"sha": "b" * 40}}}
        original_read = Path.read_text

        def require_encoding(path, *args, **kwargs):
            if kwargs.get("encoding") != "utf-8":
                raise UnicodeDecodeError("charmap", b"\x81", 0, 1, "legacy default cannot decode event")
            return original_read(path, *args, **kwargs)

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); source = root / "event.json"; output = root / "receipt.json"
            source.write_text(json.dumps(event, ensure_ascii=False), encoding="utf-8")
            env = {**env, "GITHUB_SHA": "c" * 40, "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_EVENT_PATH": str(source),
                   "GITHUB_WORKFLOW_REF": "farm-fe/farm/.github/workflows/rust-test.yaml@refs/pull/2329/merge", "CI_IMPACT_CELL": "rust-tests:ci-test:win32-x64-msvc", "CI_IMPACT_JOB_STATUS": "success"}
            with mock.patch.dict(receipt.os.environ, env, clear=True), mock.patch.object(receipt.sys, "argv", ["receipt", "--output", str(output)]), mock.patch.object(receipt.impact, "git", side_effect=[("c" * 40).encode(), ("d" * 40).encode()]), mock.patch.object(Path, "read_text", require_encoding):
                receipt.main()
            self.assertEqual(json.loads(output.read_text(encoding="utf-8"))["status"], "executed-success")

    def test_unavailable_receipt_logs_safe_operation_and_exception_type(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); source = root / "event.json"; output = root / "receipt.json"
            source.write_bytes(b'{"private-event-secret":\xff}')
            environment = {"CI_IMPACT_CELL": "cargo-check", "GITHUB_EVENT_PATH": str(source)}
            stderr = io.StringIO()
            with mock.patch.dict(receipt.os.environ, environment, clear=True), mock.patch.object(receipt.sys, "argv", ["receipt", "--output", str(output)]), contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit) as stopped:
                receipt.main()
            value = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(stopped.exception.code, 1)
            self.assertEqual(value["status"], "unavailable")
            self.assertEqual(value["error"]["stage"], "read-event")
            self.assertEqual(value["error"]["class"], "UnicodeDecodeError")
            self.assertIn("UnicodeDecodeError", stderr.getvalue())
            self.assertNotIn("private-event-secret", stderr.getvalue())
            self.assertNotIn("private-event-secret", output.read_text(encoding="utf-8"))

    def test_git_failure_logs_only_stage_and_return_code(self):
        for stage, results in [("git-head", []), ("git-tree", [("c" * 40).encode()])]:
            with self.subTest(stage=stage), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); source = root / "event.json"; output = root / "receipt.json"
                source.write_text('{}', encoding="utf-8")
                failure = subprocess.CalledProcessError(128, ["git", "secret-token"], output=b"secret-token", stderr=b"secret-token")
                stderr = io.StringIO()
                with mock.patch.dict(receipt.os.environ, {"CI_IMPACT_CELL": "cargo-check", "GITHUB_EVENT_PATH": str(source), "SECRET_TOKEN": "secret-token"}, clear=True), mock.patch.object(receipt.sys, "argv", ["receipt", "--output", str(output)]), mock.patch.object(receipt.impact, "git", side_effect=[*results, failure]), contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit):
                    receipt.main()
                value = json.loads(output.read_text(encoding="utf-8"))
                self.assertEqual(value["error"]["stage"], stage)
                self.assertEqual(value["error"]["returnCode"], 128)
                self.assertNotIn("secret-token", stderr.getvalue() + output.read_text(encoding="utf-8"))

    def test_json_and_write_failures_do_not_echo_sensitive_messages(self):
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            detail = receipt.report_error("read-event", json.JSONDecodeError("secret-token", '{"secret-token":', 2))
            receipt.report_error("validate-identity", ValueError("secret-token"))
            receipt.report_error("write-receipt", PermissionError(13, "secret-token"))
        self.assertEqual(detail["code"], "invalid-json")
        self.assertEqual(detail["line"], 1)
        self.assertNotIn("secret-token", stderr.getvalue())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); source = root / "event.json"; output = root / "receipt.json"
            source.write_text('{"secret-token":', encoding="utf-8")
            with mock.patch.dict(receipt.os.environ, {"CI_IMPACT_CELL": "cargo-check", "GITHUB_EVENT_PATH": str(source)}, clear=True), mock.patch.object(receipt.sys, "argv", ["receipt", "--output", str(output)]), contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit):
                receipt.main()
            self.assertEqual(json.loads(output.read_text(encoding="utf-8"))["error"]["code"], "invalid-json")
            self.assertNotIn("secret-token", stderr.getvalue() + output.read_text(encoding="utf-8"))
            source.write_text('{}', encoding="utf-8")
            with mock.patch.dict(receipt.os.environ, {"CI_IMPACT_CELL": "cargo-check", "GITHUB_EVENT_PATH": str(source)}, clear=True), mock.patch.object(receipt.sys, "argv", ["receipt", "--output", str(output)]), mock.patch.object(receipt.impact, "git", side_effect=ValueError("secret-token")), mock.patch.object(Path, "write_text", side_effect=PermissionError(13, "secret-token")), contextlib.redirect_stderr(stderr), self.assertRaises(SystemExit):
                receipt.main()
            self.assertIn('"stage": "write-receipt"', stderr.getvalue())
            self.assertNotIn("secret-token", stderr.getvalue())

    def test_summary_collects_all_unavailable_cells_without_echoing_old_errors(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for index, cell in enumerate(["cargo-check", "rust-tests:ci-test:win32-x64-msvc"]):
                error = {"stage": "read-event", "class": "UnicodeDecodeError", "code": "invalid-text-encoding", "encoding": "utf-8", "runId": "10", "runAttempt": "2", "message": "secret-token"} if index == 0 else "secret-token"
                (root / (str(index) + '.json')).write_text(json.dumps({"cell": cell, "status": "unavailable", "error": error}), encoding="utf-8")
            rows, unavailable = summary.read_observations(root)
            self.assertEqual(len(rows), 2)
            self.assertEqual(len(unavailable), 2)
            self.assertEqual({row["cell"] for row in unavailable}, {"cargo-check", "rust-tests:ci-test:win32-x64-msvc"})
            self.assertNotIn("secret-token", json.dumps(unavailable))

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
