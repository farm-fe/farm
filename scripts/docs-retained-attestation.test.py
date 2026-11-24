# cspell:words importlib
import base64
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("docs_signing", Path(__file__).with_name("docs-retained-attestation.py"))
policy = importlib.util.module_from_spec(spec); spec.loader.exec_module(policy)


class SignerPolicy(unittest.TestCase):
    def setUp(self):
        self.expected = {"repositoryId": 507542208, "signerWorkflow": ".github/workflows/docs-retention-signer.yaml",
                         "callerWorkflow": ".github/workflows/docs-public-retention.yaml", "signerSHA": "1" * 40,
                         "originalSourceSHA": "2" * 40, "originalSourceTree": "3" * 40, "runId": 1234, "attempt": 2}
        self.receipt = {"schemaVersion": 1, "kind": policy.KIND, "sourceSHA": "2" * 40, "sourceTree": "3" * 40,
                        "packages": [{"name": name, "version": "2.0.0", "archiveSha256": "a" * 64, "archiveBytes": 1,
                                      "integrity": "sha512-" + base64.b64encode(b"a" * 64).decode("ascii"),
                                      "resolved": f"https://registry.npmjs.org/{name}/-/{name.split('/')[1]}-2.0.0.tgz",
                                      "metadataSha256": "b" * 64} for name in policy.PAIR]}
        self.expected["frozenArtifact"] = {"runId": 2345, "attempt": 1, "artifactId": 31, "artifactSha256": "c" * 64}
        self.expected["originalArchives"] = {
            row["name"]: {field: row[field] for field in policy.ARCHIVE_FIELDS} for row in self.receipt["packages"]
        }
        self.calls = []
        self.overrides = {}

    def cli(self, body, bundle, argv):
        self.calls.append((body, bundle))
        self.assertEqual(argv[argv.index("--source-digest") + 1], "1" * 40)
        self.assertIn("--deny-self-hosted-runners", argv)
        signer = "https://github.com/farm-fe/farm/.github/workflows/docs-retention-signer.yaml@refs/heads/main"
        return [{"verificationResult": {"signature": {"certificate": {
            "issuer": policy.ISSUER, "subjectAlternativeName": signer, "buildSignerURI": signer,
            "buildSignerDigest": "1" * 40, "runnerEnvironment": "github-hosted",
            "sourceRepositoryURI": "https://github.com/farm-fe/farm", "sourceRepositoryIdentifier": "507542208",
            "sourceRepositoryDigest": "1" * 40, "sourceRepositoryRef": "refs/heads/main",
            "buildConfigURI": "https://github.com/farm-fe/farm/.github/workflows/docs-public-retention.yaml@refs/heads/main",
            "buildConfigDigest": "1" * 40, "buildTrigger": "push",
            "runInvocationURI": "https://github.com/farm-fe/farm/actions/runs/1234/attempts/2", **self.overrides}},
            "statement": {"_type": "https://in-toto.io/Statement/v1", "predicateType": policy.PREDICATE,
                          "predicate": {"schemaVersion": 1, "kind": policy.KIND, "frozenArtifact": dict(self.expected["frozenArtifact"])},
                          "subject": [{"name": policy.SUBJECT, "digest": {"sha256": hashlib.sha256(body).hexdigest()}}]}}}]

    def verify(self, *, cli=None, body=None):
        return policy.verify_retained_attestation(body or json.dumps(self.receipt).encode(), b"bundle",
                                                  expected=self.expected, verify=cli or self.cli)

    def test_repacked_public_tarball_does_not_reach_verifier(self):
        original = copy.deepcopy(self.receipt)
        for key, value in (("archiveSha256", "f" * 64), ("archiveBytes", 2),
                           ("integrity", "sha512-" + base64.b64encode(b"b" * 64).decode("ascii"))):
            self.receipt = copy.deepcopy(original)
            self.receipt["packages"][0][key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "original archive"):
                self.verify()
        self.assertEqual(self.calls, [])

    def test_frozen_artifact_replay_and_wrong_attempt_refused(self):
        def wrong_artifact(body, bundle, argv):
            out = self.cli(body, bundle, argv)
            out[0]["verificationResult"]["statement"]["predicate"]["frozenArtifact"]["artifactId"] = 32
            return out
        with self.assertRaisesRegex(ValueError, "subject/predicate"):
            self.verify(cli=wrong_artifact)
        original = dict(self.expected["frozenArtifact"])
        for patch in ({"attempt": 2}, {"runId": self.expected["runId"]}, {"artifactSha256": "short"}):
            self.expected["frozenArtifact"] = {**original, **patch}
            with self.subTest(patch=patch), self.assertRaisesRegex(ValueError, "first-attempt artifact"):
                self.verify()

    def test_original_source_may_differ_from_later_trusted_signer(self):
        self.assertEqual(self.verify(), self.receipt)

    def test_wrong_source_partial_duplicate_or_extra_authority_never_invokes_cli(self):
        original = copy.deepcopy(self.receipt)
        for patch in ({"sourceTree": "4" * 40}, {"sourceSHA": "1" * 40}, {"packages": original["packages"][:1]},
                      {"packages": original["packages"][:1] * 2}, {"publish": True}):
            with self.subTest(patch=patch), self.assertRaises(ValueError):
                self.receipt = {**original, **patch}
                self.verify()
        body = json.dumps(original).replace('"schemaVersion": 1', '"schemaVersion": 1, "schemaVersion": 1')
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            self.verify(body=body.encode())
        self.assertEqual(self.calls, [])

    def test_wrong_signer_caller_ref_event_run_attempt_runner_and_repositories_refused(self):
        for field, value in (("issuer", "https://example.com"), ("buildSignerDigest", "2" * 40),
                             ("buildConfigURI", "https://example.com"), ("sourceRepositoryRef", "refs/heads/topic"),
                             ("buildTrigger", "workflow_dispatch"), ("runInvocationURI", "https://example.com"),
                             ("runnerEnvironment", "self-hosted"), ("sourceRepositoryIdentifier", "other")):
            self.overrides = {field: value}
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, "certificate identity"):
                self.verify()

    def test_wrong_subject_digest_name_predicate_or_multiple_verifications_refused(self):
        for kind in ("digest", "name", "predicate", "two"):
            def verify(body, bundle, argv):
                out = self.cli(body, bundle, argv)
                statement = out[0]["verificationResult"]["statement"]
                if kind == "two":
                    return out * 2
                if kind == "predicate":
                    statement["predicateType"] = "https://farmfe.org/attestations/ci-bindings/v1"
                elif kind == "name":
                    statement["subject"][0]["name"] = "ci-bindings.json"
                else:
                    statement["subject"][0]["digest"]["sha256"] = "0" * 64
                return out
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.verify(cli=verify)


if __name__ == "__main__":
    unittest.main()
