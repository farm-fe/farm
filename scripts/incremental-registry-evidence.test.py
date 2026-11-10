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


class BudgetContracts(unittest.TestCase):
    def fixture(self, open_cost=0, read_cost=0):
        now = [0.0]; calls = []
        class Response(io.BytesIO):
            def read(self, count=-1):
                now[0] += read_cost
                return super().read(count)
        class Opener:
            def open(self, request, timeout):
                calls.append(timeout); now[0] += open_cost
                return Response(b'{}')
        client = registry.RegistryReadOnly({'GITHUB_ACTIONS': 'true'}, Opener(), lambda: now[0])
        return client, now, calls

    def test_non_registry_work_does_not_expire_next_operation(self):
        client, now, calls = self.fixture()
        now[0] = 601
        self.assertEqual(client.json('https://registry.npmjs.org/a'), {})
        now[0] += 601
        self.assertEqual(client.json('https://registry.npmjs.org/b'), {})
        self.assertEqual(calls, [30, 30])

    def test_request_and_slow_metadata_body_have_deadlines(self):
        for opening, reading in [(31, 0), (0, 31)]:
            client, _, calls = self.fixture(opening, reading)
            with self.assertRaisesRegex(ValueError, 'deadline'):
                client.json('https://registry.npmjs.org/a')
            self.assertEqual(calls, [30])

    def test_nested_operations_do_not_extend_batch_deadline(self):
        client, now, calls = self.fixture()
        with self.assertRaisesRegex(ValueError, 'deadline'):
            with client.budget(600):
                now[0] = 599
                self.assertEqual(client.json('https://registry.npmjs.org/a'), {})
                now[0] = 600
                client.json('https://registry.npmjs.org/b')
        self.assertEqual(calls, [1])
        # A failed operation does not leak its deadline into a later operation.
        self.assertEqual(client.json('https://registry.npmjs.org/c'), {})

    def test_slow_stream_cannot_refresh_its_deadline(self):
        client, _, _ = self.fixture(read_cost=601)
        with self.assertRaisesRegex(ValueError, 'deadline'):
            client.archive('https://registry.npmjs.org/a.tgz')
        self.assertEqual(client.total_bytes, 2)

    def test_single_archive_limit_and_invalid_budget_are_retained(self):
        client, _, _ = self.fixture()
        class Chunks(io.BytesIO):
            def read(self, count=-1): return b'x' * (1024 * 1024)
        class Opener:
            def open(self, request, timeout): return Chunks()
        client.opener = Opener()
        with self.assertRaisesRegex(ValueError, 'byte budget'):
            client.archive('https://registry.npmjs.org/a.tgz')
        self.assertEqual(client.total_bytes, 101 * 1024 * 1024)
        for value in [0, -1, True, 4501, float('inf')]:
            with self.assertRaisesRegex(ValueError, 'Invalid registry operation budget'):
                with client.budget(value): pass

    def test_byte_budget_survives_operation_boundaries(self):
        client, now, _ = self.fixture()
        client.total_bytes = 3 * 1024 ** 3 - 3
        client.archive('https://registry.npmjs.org/a.tgz')
        now[0] += 601
        with self.assertRaisesRegex(ValueError, 'byte budget'):
            client.archive('https://registry.npmjs.org/b.tgz')
        self.assertEqual(client.total_bytes, 3 * 1024 ** 3 + 1)


if __name__ == "__main__":
    unittest.main()
