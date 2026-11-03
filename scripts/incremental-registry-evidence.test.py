"""Public registry byte verification contracts; no actual network in these tests."""
# cspell:words importlib npmjs
import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("registry_contract", Path(__file__).with_name("incremental-registry-evidence.py"))
registry = importlib.util.module_from_spec(spec)
spec.loader.exec_module(registry)


class RegistryContracts(unittest.TestCase):
    def client(self, routes):
        class Opener:
            def open(self, request, timeout):
                if dict(request.header_items()) != {"Accept": "application/json", "User-agent": "Farm-release-read-only-verifier"}:
                    raise AssertionError("Unexpected registry headers")
                return io.BytesIO(routes[request.full_url])
        return registry.RegistryReadOnly({"GITHUB_ACTIONS": "true"}, Opener())

    def test_scoped_npm_tarball_is_hashed_independently_from_ledger_fields(self):
        body = b'original immutable public npm archive'
        url = "https://registry.npmjs.org/@farmfe/a/-/a-1.0.0.tgz"
        metadata = {"name": "@farmfe/a", "version": "1.0.0", "dist": {"tarball": url,
                    "integrity": "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode("ascii")}}
        routes = {"https://registry.npmjs.org/%40farmfe%2Fa/1.0.0": json.dumps(metadata).encode(), url: body}
        key = "npm:@farmfe/a"; row = {"version": "1.0.0", "archiveSha256": hashlib.sha256(body).hexdigest()}
        self.assertEqual(self.client(routes).verify({key: row}), {key: row["archiveSha256"]})
        row["archiveSha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "Published archive differs"):
            self.client(routes).verify({key: row})

    def test_cargo_metadata_and_original_archive_must_both_match(self):
        body = b'original immutable Cargo archive'; checksum = hashlib.sha256(body).hexdigest()
        routes = {"https://crates.io/api/v1/crates/farmfe_core/1.0.0": json.dumps({"version": {"crate": "farmfe_core", "num": "1.0.0", "checksum": checksum}}).encode(),
                  "https://static.crates.io/crates/farmfe_core/farmfe_core-1.0.0.crate": body}
        self.assertEqual(self.client(routes).verify({"cargo:farmfe_core": {"version": "1.0.0", "archiveSha256": checksum}}), {"cargo:farmfe_core": checksum})
        routes["https://static.crates.io/crates/farmfe_core/farmfe_core-1.0.0.crate"] = b'different public bytes'
        with self.assertRaisesRegex(ValueError, "Cargo archive checksum"):
            self.client(routes).verify({"cargo:farmfe_core": {"version": "1.0.0", "archiveSha256": checksum}})

    def test_transport_refuses_local_execution_credentials_and_other_origins(self):
        with self.assertRaises(PermissionError):
            registry.RegistryReadOnly({})
        client = self.client({})
        for url in ["http://registry.npmjs.org/a", "https://registry.npmjs.org.evil.invalid/a", "https://secret@registry.npmjs.org/a"]:
            with self.assertRaises(ValueError):
                client.open(url)


if __name__ == "__main__":
    unittest.main()
