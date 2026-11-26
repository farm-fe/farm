"""Offline contracts for a manually reviewed exact Docs upload permit."""
# cspell:words importlib
from copy import deepcopy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("permit", Path(__file__).with_name("docs-pair-owner-approval.py"))
permit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(permit)


class Contract(unittest.TestCase):
    def setUp(self):
        self.expected = {"artifactId": 10, "artifactSha256": "a"*64,
                         "archives": dict(zip(permit.PACKAGE_ORDER, ["b"*64, "c"*64])),
                         "environmentId": 11, "sourceSHA": "d"*40, "sourceTree": "e"*40,
                         "producerRun": 12, "writerRun": 13, "attempt": 1, "tag": "farm-docs-bootstrap"}
        self.review = {"state": "approved", "comment": permit.approval_comment(self.expected),
                       "environments": [{"id": 11, "name": "docs-pair-production"}], "user": {"id": 5}}

    def verify(self, expected=None, review=None, actor=6):
        return permit.verify_owner_review(self.expected if expected is None else expected,
            review_history=lambda route: self.history(route, [self.review] if review is None else review),
            trusted_reviewer_ids=frozenset({5}), actor_id=actor, environment_name="docs-pair-production")

    def history(self, route, records):
        self.assertEqual(route, "/actions/runs/13/approvals")
        return records

    def test_exact_machine_comment_is_read_only_and_key_order_independent(self):
        self.assertTrue(self.verify(dict(reversed(list(self.expected.items())))))
        self.assertRegex(self.review["comment"], r"^farm-docs-2\.0\.0 approve sha256:[0-9a-f]{64}$")

    def test_report_cannot_substitute_approval_for_other_bytes(self):
        for key, value in [("artifactSha256", "f"*64), ("artifactId", 14), ("sourceTree", "f"*40),
                           ("sourceSHA", "f"*40), ("producerRun", 15),
                           ("archives", dict(zip(permit.PACKAGE_ORDER, ["c"*64, "b"*64])))]:
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "exact first release"):
                self.verify({**self.expected, key: value})

    def test_attempt_replay_and_stable_tag_deny_before_any_api_read(self):
        for key, value in [("attempt", True), ("attempt", 2), ("writerRun", 12), ("tag", "latest")]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify({**self.expected, key: value})
        new = {**self.expected, "writerRun": 14}
        self.assertNotEqual(permit.approval_comment(new), self.review["comment"])
        def stale(route):
            self.assertEqual(route, "/actions/runs/14/approvals")
            return [self.review]
        with self.assertRaisesRegex(ValueError, "exact first release"):
            permit.verify_owner_review(new, review_history=stale, actor_id=6,
                trusted_reviewer_ids=frozenset({5}), environment_name="docs-pair-production")

    def test_actor_allowlist_and_no_self_review(self):
        with self.assertRaisesRegex(ValueError, "separate approved"):
            self.verify(actor=5)
        with self.assertRaisesRegex(ValueError, "separate approved"):
            self.verify(review=[{**self.review, "user": {"id": 7}}])

    def test_empty_duplicate_rejected_and_wrong_environment_deny(self):
        candidates = ([], [self.review, deepcopy(self.review)], [{**self.review, "state": "rejected"}],
                      [{**self.review, "environments": [{"id": 10, "name": "docs-pair-production"}]}],
                      [{**self.review, "environments": [{"id": 11, "name": "docs-pair-development"}]}])
        for records in candidates:
            with self.subTest(records=records), self.assertRaises(ValueError):
                self.verify(review=records)


if __name__ == "__main__":
    unittest.main()
