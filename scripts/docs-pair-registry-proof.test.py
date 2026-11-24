"""Injected registry tests for the Docs pair adoption boundary."""
# cspell:words importlib
import base64
import copy
import hashlib
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("docs_pair", Path(__file__).with_name("docs-pair-registry-proof.py"))
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)


class Contract(unittest.TestCase):
    def setUp(self):
        self.sha, self.tree = "a" * 40, "b" * 40
        self.calls = []
        common = {"schemaVersion": 1, "sourceSHA": self.sha, "sourceTree": self.tree}
        self.expected = {**common, "kind": "docs-frozen-pair-v1", "packages": []}
        self.installed = {**common, "kind": "docs-public-install-v1", "packages": []}
        self.public = {}
        for name in proof.ORDER:
            body = name.encode("utf-8")
            row = {"name": name, "version": proof.VERSION, "archiveBytes": len(body),
                   "archiveSha256": hashlib.sha256(body).hexdigest(),
                   "integrity": "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode("ascii")}
            filename = name.split("/")[1]
            url = f"https://registry.npmjs.org/{name}/-/{filename}-{proof.VERSION}.tgz"
            self.expected["packages"].append(row)
            self.installed["packages"].append({**row, "resolved": url})
            metadata = {field: {} for field in proof.DEP_FIELDS}
            if name == proof.THEME:
                metadata["peerDependencies"][proof.DOCS] = proof.VERSION
            else:
                metadata["dependencies"][proof.THEME] = proof.VERSION
            self.public[name] = {**row, "status": "published", "tarballURL": url, "metadata": metadata}

    def lookup(self, row, *, verify_archive):
        self.assertEqual(row["kind"], "npm")
        self.assertEqual(row["version"], proof.VERSION)
        self.calls.append((row["name"], verify_archive))
        out = copy.deepcopy(self.public[row["name"]])
        if not verify_archive:
            out.pop("archiveBytes", None)
            out.pop("archiveSha256", None)
        out["evidenceKind"] = "registry-archive" if verify_archive else "registry-metadata"
        return out

    def verify(self, *, lookup=None):
        return proof.verify_pair(self.expected, self.installed, expected_source_sha=self.sha,
                                 expected_source_tree=self.tree, lookup=lookup or self.lookup)

    def test_exact_measured_pair_adopts_only_after_both_reads(self):
        result = self.verify()
        self.assertEqual(result["kind"], "docs-public-retain-only-v1")
        self.assertEqual(result["sourceTree"], self.tree)
        self.assertEqual(self.calls, [(name, mode) for mode in (False, True) for name in proof.ORDER])
        self.assertEqual([row["name"] for row in result["packages"]], list(proof.ORDER))
        self.assertTrue(all(len(row["metadataSha256"]) == 64 for row in result["packages"]))

    def test_malformed_or_untrusted_install_does_not_query_registry(self):
        original = copy.deepcopy(self.installed)
        for field, value in [("name", "other"), ("version", "2.0.1"), ("archiveSha256", "0" * 64),
                             ("archiveBytes", True), ("integrity", "sha512-A==="),
                             ("resolved", "file:../../scratch.tgz"),
                             ("resolved", "https://registry.npmjs.org.example.com/other")]:
            with self.subTest(field=field, value=value):
                self.installed = copy.deepcopy(original)
                self.installed["packages"][0][field] = value
                with self.assertRaises(ValueError):
                    self.verify()
                self.assertEqual(self.calls, [])
        self.installed = original
        self.expected["sourceTree"] = "c" * 40
        with self.assertRaisesRegex(ValueError, "source differs"):
            self.verify()
        self.assertEqual(self.calls, [])

    def test_missing_duplicate_and_lookalike_receipts_never_query_registry(self):
        original = self.expected["packages"]
        for rows in (original[:1], original + original[:1], original[:1] * 2):
            with self.subTest(count=len(rows)):
                self.expected["packages"] = rows
                with self.assertRaises(ValueError):
                    self.verify()
                self.assertEqual(self.calls, [])
        self.expected["packages"] = original
        self.expected["schemaVersion"] = True
        with self.assertRaises(ValueError):
            self.verify()
        self.assertEqual(self.calls, [])

    def test_second_not_public_never_downloads_or_adopts_first(self):
        for status in ("missing", "unknown"):
            with self.subTest(status=status):
                self.public[proof.DOCS]["status"] = status
                self.calls.clear()
                with self.assertRaisesRegex(ValueError, "not fully public"):
                    self.verify()
                self.assertEqual(self.calls, [(name, False) for name in proof.ORDER])

    def test_mutated_bytes_under_matching_registry_sri_are_not_adopted(self):
        self.public[proof.DOCS]["archiveSha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "archive bytes"):
            self.verify()
        self.assertEqual(self.calls[-1], (proof.DOCS, True))

    def test_exact_required_edges_and_same_metadata_after_download(self):
        self.public[proof.THEME]["metadata"]["peerDependenciesMeta"][proof.DOCS] = {"optional": True}
        with self.assertRaisesRegex(ValueError, "exact and mandatory"):
            self.verify()
        self.public[proof.THEME]["metadata"]["peerDependenciesMeta"].clear()
        def split(row, *, verify_archive):
            observed = self.lookup(row, verify_archive=verify_archive)
            if verify_archive:
                observed["metadata"]["dependencies"]["irrelevant"] = "1.0.0"
            return observed
        with self.assertRaisesRegex(ValueError, "metadata changed"):
            self.verify(lookup=split)

    def test_metadata_only_claim_and_hostile_tgz_route_fail_before_download(self):
        def split(row, *, verify_archive):
            observed = self.lookup(row, verify_archive=verify_archive)
            observed["archiveSha256"] = "0" * 64
            return observed
        with self.assertRaisesRegex(ValueError, "claims measured"):
            self.verify(lookup=split)
        self.public[proof.THEME]["tarballURL"] += "?mutable=true"
        self.calls.clear()
        with self.assertRaisesRegex(ValueError, "URL or SRI"):
            self.verify()
        self.assertEqual(self.calls, [(proof.THEME, False)])


if __name__ == "__main__":
    unittest.main()
