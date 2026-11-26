# cspell:words USERCONFIG
"""Local fake subprocess contracts; no network or actual npm invocation."""
# cspell:words importlib tmpdir
import base64
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
from types import SimpleNamespace
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("docs_pair_transport", Path(__file__).with_name("docs-pair-npm-transport.py"))
transport = importlib.util.module_from_spec(spec)
spec.loader.exec_module(transport)
proof = transport.proof


def manifest(name):
    declaration = {"dependencies": {proof.THEME: proof.VERSION}} if name == proof.DOCS else {
        "peerDependencies": {proof.DOCS: proof.VERSION}}
    return {"name": name, "version": proof.VERSION, "repository": "https://github.com/farm-fe/farm", **declaration}


def archive(name):
    data = json.dumps(manifest(name)).encode()
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as opened:
        item = tarfile.TarInfo("package/package.json")
        item.size = len(data)
        opened.addfile(item, io.BytesIO(data))
    body = output.getvalue()
    return body, {"name": name, "version": proof.VERSION, "archiveBytes": len(body),
                  "archiveSha256": hashlib.sha256(body).hexdigest(),
                  "integrity": "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode()}


class Contracts(unittest.TestCase):
    def setUp(self):
        self.rows, self.bodies = {}, {}
        for name in proof.ORDER:
            self.bodies[name], self.rows[name] = archive(name)
        self.calls, self.tokens = [], []
        self.bad, self.fail_publish = "", False
        self.exec = transport.NpmExecution("/trusted/node", "/trusted/npm", run=self.run_npm)

    def token(self, name):
        self.tokens.append(name)
        return "mock-value"

    def run_npm(self, command, *, cwd, env, stdin, capture_output, timeout, check):
        self.assertEqual(command[:2], ["/trusted/node", "/trusted/npm"])
        self.assertEqual(stdin, subprocess.DEVNULL)
        self.assertTrue(capture_output)
        self.assertFalse(check)
        self.assertNotIn("GH_TOKEN", env)
        self.assertNotIn("NPM_TOKEN", env)
        root = Path(cwd)
        self.assertEqual(Path(env["HOME"]), root)
        self.assertEqual(env["npm_config_fetch_retries"], "0")
        args = command[2:]
        verb = args[0]
        self.calls.append(verb)
        if verb == "--version":
            self.assertNotIn("NODE_AUTH_TOKEN", env)
            return SimpleNamespace(returncode=0, stdout=b"11.21.2\n")
        self.assertIn("--ignore-scripts", args)
        self.assertIn("--fetch-retries=0", args)
        self.assertIn("--registry=https://registry.npmjs.org/", args)
        if verb == "publish":
            self.assertEqual(env["NODE_AUTH_TOKEN"], "mock-value")
            self.assertNotIn("mock-value", " ".join(command))
            self.assertEqual(args[args.index("--tag") + 1], "farm-docs-bootstrap")
            self.assertIn("${NODE_AUTH_TOKEN}", Path(env["NPM_CONFIG_USERCONFIG"]).read_text())
            self.assertIn(Path(args[1]).read_bytes(), self.bodies.values())
            return SimpleNamespace(returncode=int(self.fail_publish), stdout=b"raw-secret-never-echoed")
        self.assertNotIn("NODE_AUTH_TOKEN", env)
        self.assertNotIn("_authToken", Path(env["NPM_CONFIG_USERCONFIG"]).read_text())
        dependencies = json.loads((root / "package.json").read_text())["dependencies"]
        if verb == "install":
            pkgs = {"": {"dependencies": dependencies}}
            for name in proof.ORDER:
                filename = name.split("/")[1]
                resolution = dependencies[name] if dependencies[name].startswith("file:") else (
                    f"https://registry.npmjs.org/{name}/-/{filename}-2.0.0.tgz")
                pkgs["node_modules/" + name] = {"version": "2.0.0", "integrity": self.rows[name]["integrity"],
                                               "resolved": resolution}
            if self.bad == "lock-integrity":
                pkgs["node_modules/" + proof.THEME]["integrity"] = self.rows[proof.DOCS]["integrity"]
            if self.bad == "host":
                pkgs["node_modules/rogue"] = {"integrity": self.rows[proof.DOCS]["integrity"],
                                             "resolved": "https://registry.npmjs.org.example/rogue.tgz"}
            if self.bad == "nested":
                pkgs["node_modules/rogue/node_modules/" + proof.DOCS] = pkgs["node_modules/" + proof.DOCS]
            (root / "package-lock.json").write_text(json.dumps({"lockfileVersion": 3, "packages": pkgs}))
        else:
            self.assertEqual(verb, "ci")
            for name in proof.ORDER:
                package = root / "node_modules" / name
                package.mkdir(parents=True)
                contents = manifest(name)
                if name == proof.THEME and self.bad == "manifest":
                    contents["peerDependencies"][proof.DOCS] = "*"
                (package / "package.json").write_text(json.dumps(contents))
            if self.bad == "lock-swap":
                with (root / "package-lock.json").open("a") as stream:
                    stream.write(" ")
        return SimpleNamespace(returncode=0, stdout=b"")

    def consumer(self, **kwargs):
        with mock.patch.dict(os.environ, {"GH_TOKEN": "inherited", "NODE_AUTH_TOKEN": "inherited"}):
            return self.exec.verify_installed(self.rows, source_sha="a" * 40, source_tree="b" * 40, **kwargs)

    def test_exact_publish_and_nonzero_never_retries(self):
        self.fail_publish = True
        with self.assertRaisesRegex(ValueError, "unconfirmed; do not rerun"):
            self.exec.publish(self.rows[proof.THEME], self.bodies[proof.THEME], token_supplier=self.token)
        with self.assertRaisesRegex(ValueError, "one-shot"):
            self.exec.publish(self.rows[proof.THEME], self.bodies[proof.THEME], token_supplier=self.token)
        self.assertEqual(self.calls, ["--version", "publish"])
        self.assertEqual(self.tokens, ["NPM_BYPASS_2FA_TOKEN"])

    def test_wrong_tag_sri_or_frozen_body_never_invokes_subprocess_or_supplies_token(self):
        row, body = self.rows[proof.THEME], self.bodies[proof.THEME]
        checks = [(row, body, "latest"), ({**row, "integrity": self.rows[proof.DOCS]["integrity"]}, body, transport.STAGING_TAG),
                  (row, self.bodies[proof.DOCS], transport.STAGING_TAG)]
        for record, contents, tag in checks:
            with self.assertRaises(ValueError):
                self.exec.publish(record, contents, tag=tag, token_supplier=self.token)
        self.assertEqual(self.calls, [])
        self.assertEqual(self.tokens, [])

    def test_validate_both_frozen_archives_before_any_token_or_subprocess(self):
        manifests = transport.validate_pair(self.rows, self.bodies)
        self.assertEqual(set(manifests), set(proof.ORDER))
        with self.assertRaises(ValueError):
            transport.validate_pair(self.rows, {**self.bodies, proof.DOCS: b"invalid"})
        self.assertEqual(self.calls, [])
        self.assertEqual(self.tokens, [])

    def test_midpoint_and_all_public_consumer_use_exact_lock_without_token(self):
        self.assertIs(self.consumer(local_docs=self.bodies[proof.DOCS]), True)
        receipt = self.consumer()
        self.assertEqual(receipt["kind"], "docs-public-install-v1")
        self.assertEqual([row["name"] for row in receipt["packages"]], list(proof.ORDER))
        self.assertTrue(all(row["resolved"].startswith(transport.REGISTRY) for row in receipt["packages"]))
        self.assertEqual(self.calls, ["--version", "install", "ci"] * 2)

    def test_bad_lock_or_nested_duplicate_stops_before_install(self):
        for bad in ("lock-integrity", "host", "nested"):
            with self.subTest(bad=bad):
                self.calls.clear()
                self.bad = bad
                with self.assertRaises(ValueError):
                    self.consumer()
                self.assertEqual(self.calls, ["--version", "install"])

    def test_mutated_lock_or_bad_installed_peer_rejected(self):
        for bad in ("lock-swap", "manifest"):
            with self.subTest(bad=bad):
                self.bad = bad
                with self.assertRaises(ValueError):
                    self.consumer()


if __name__ == "__main__":
    unittest.main()
