# cspell:words WRONLY CREAT USERCONFIG GLOBALCONFIG
"""Exact npm subprocess boundary for the reviewed two-package Docs handoff."""
# cspell:words importlib npmrc
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import urllib.parse


def sibling(file):
    spec = importlib.util.spec_from_file_location(file, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


proof = sibling("docs-pair-registry-proof.py")
tarball = sibling("release-npm-tarball.py")
require = proof.require
TOKEN_NAME = "NPM_BYPASS_2FA_TOKEN"
STAGING_TAG = "farm-docs-bootstrap"
REGISTRY = "https://registry.npmjs.org/"
NPM_VERSION = b"11.21.2"
ROW_FIELDS = {"name", "version", "archiveSha256", "archiveBytes", "integrity"}
FLAGS = ["--ignore-scripts", "--no-audit", "--no-fund", "--fetch-retries=0",
         "--replace-registry-host=never", "--registry=https://registry.npmjs.org/"]


def exact_rows(rows, source_sha, source_tree):
    require(proof.identity(source_sha, 40) and proof.identity(source_tree, 40)
            and type(rows) is dict and set(rows) == set(proof.ORDER), "Exact source and package rows required")
    checked = proof.package_rows({"schemaVersion": 1, "kind": "docs-frozen-pair-v1",
                                  "sourceSHA": source_sha, "sourceTree": source_tree,
                                  "packages": list(rows.values())},
                                 kind="docs-frozen-pair-v1", source_sha=source_sha, source_tree=source_tree)
    require(all(rows[name] == checked[name] for name in proof.ORDER), "Docs row key differs")
    return {name: dict(checked[name]) for name in proof.ORDER}


def inspect_archive(row, body):
    require(type(row) is dict and set(row) == ROW_FIELDS
            and row["name"] in proof.ORDER and row["version"] == proof.VERSION
            and type(body) is bytes and type(row["archiveBytes"]) is int
            and 0 < len(body) == row["archiveBytes"] <= 32 * 1024**2
            and proof.integrity(row["integrity"])
            and row["integrity"] == "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode("ascii"),
            "Frozen Docs archive identity differs")
    manifest = tarball.validate({"kind": "npm", **row}, body)
    require(not (set(manifest.get("publishConfig", {})) & {"tag", "provenance"}),
            "Docs archive may not select its own tag or provenance")
    proof.metadata_for(row["name"], {field: manifest.get(field, {}) for field in proof.DEP_FIELDS})
    return manifest


def validate_pair(rows, archives):
    """Validate both originals before either can reach a protected writer."""
    require(type(rows) is dict and type(archives) is dict
            and set(rows) == set(archives) == set(proof.ORDER), "Complete frozen Docs pair required")
    manifests = {}
    for name in proof.ORDER:
        require(type(rows[name]) is dict and rows[name].get("name") == name, "Docs pair row key differs")
        manifests[name] = inspect_archive(rows[name], archives[name])
    return manifests


def public_tar_url(value):
    if type(value) is not str or len(value) > 2048 or any(ord(c) < 33 for c in value):
        return False
    parsed = urllib.parse.urlsplit(value)
    return (parsed.scheme == "https" and parsed.netloc == "registry.npmjs.org" and parsed.path.startswith("/")
            and not parsed.query and not parsed.fragment and value.startswith(REGISTRY))


def checked_json(path, root, limit):
    require(path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(root.resolve())
            and 0 < path.stat().st_size <= limit, "Invalid consumer JSON file")
    return json.loads(path.read_bytes(), object_pairs_hook=tarball.unique_object,
                      parse_constant=tarball.invalid_constant)


class NpmExecution:
    validate_pair = staticmethod(validate_pair)

    def __init__(self, node, npm_cli, *, run=subprocess.run):
        self.node, self.npm = str(node), str(npm_cli)
        require(Path(node).is_absolute() and Path(npm_cli).is_absolute(), "Controlled absolute Node/npm paths required")
        self.run = run
        self.used = set()

    def invoke(self, root, environment, args, timeout, message):
        try:
            result = self.run([self.node, self.npm, *args], cwd=str(root), env=environment,
                              stdin=subprocess.DEVNULL, capture_output=True, timeout=timeout, check=False)
        except (subprocess.SubprocessError, OSError):
            raise ValueError(message) from None
        require(result.returncode == 0, message)
        return result.stdout

    def initialize(self, root, *, auth=False):
        user, global_cfg = root / "user.npmrc", root / "global.npmrc"
        for destination, contents in ((user, "registry=" + REGISTRY + "\n"), (global_cfg, "")):
            descriptor = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
                stream.write(contents)
        environment = {"PATH": os.defpath, "HOME": str(root), "NPM_CONFIG_USERCONFIG": str(user),
                       "NPM_CONFIG_GLOBALCONFIG": str(global_cfg), "npm_config_cache": str(root / "cache"),
                       "npm_config_fetch_retries": "0", "npm_config_ignore_scripts": "true",
                       "npm_config_update_notifier": "false", "npm_config_registry": REGISTRY}
        version = self.invoke(root, environment, ["--version"], 15, "Pinned npm version check failed")
        require(isinstance(version, bytes) and version.strip() == NPM_VERSION, "Exact npm 11.21.2 required")
        if auth:
            with user.open("a", encoding="utf-8") as stream:
                stream.write("//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}\n")
        return environment

    def publish(self, row, body, *, token_supplier, before_submit, tag=STAGING_TAG):
        """Run the trusted caller's final gate after setup and before using the token."""
        require(tag == STAGING_TAG, "Only the fixed Docs staging tag is allowed")
        inspect_archive(row, body)
        name = row["name"]
        require(name not in self.used, "Docs package subprocess is one-shot")
        self.used.add(name)
        with tempfile.TemporaryDirectory(prefix="farm-docs-npm-") as directory:
            root = Path(directory)
            environment = self.initialize(root, auth=True)
            archive = root / "sealed.tgz"
            with os.fdopen(os.open(archive, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as stream:
                stream.write(body)
            before_submit()
            token = token_supplier(TOKEN_NAME)
            require(type(token) is str and 0 < len(token) <= 4096 and all(33 <= ord(c) < 127 for c in token),
                    "Explicit Docs bootstrap token missing or invalid")
            self.invoke(root, {**environment, "NODE_AUTH_TOKEN": token},
                        ["publish", str(archive), "--access", "public", "--tag", STAGING_TAG, "--provenance=false", *FLAGS],
                        120, "Docs publish outcome unconfirmed; do not rerun")
        return True

    def verify_installed(self, rows, *, source_sha, source_tree, local_docs=None):
        rows = exact_rows(rows, source_sha, source_tree)
        if local_docs is not None:
            inspect_archive(rows[proof.DOCS], local_docs)
        with tempfile.TemporaryDirectory(prefix="farm-docs-public-install-") as directory:
            root = Path(directory)
            environment = self.initialize(root)
            dependencies = {**{name: proof.VERSION for name in proof.ORDER}, "react": "18.2.0", "react-dom": "18.2.0"}
            if local_docs is not None:
                (root / "sealed-docs.tgz").write_bytes(local_docs)
                dependencies[proof.DOCS] = "file:sealed-docs.tgz"
            (root / "package.json").write_text(json.dumps({"name": "farm-docs-public-consumer", "version": "1.0.0",
                                                          "private": True, "dependencies": dependencies}), encoding="utf-8")
            self.invoke(root, environment, ["install", "--package-lock-only", *FLAGS], 360, "Clean Docs lock resolution failed")
            lock_path = root / "package-lock.json"
            lock = checked_json(lock_path, root, 16 * 1024**2)
            require(type(lock) is dict and lock.get("lockfileVersion") == 3 and type(lock.get("packages")) is dict
                    and lock["packages"].get("", {}).get("dependencies") == dependencies, "Exact clean consumer lock required")
            packages = lock["packages"]
            for package, row in packages.items():
                if not package:
                    continue
                require(type(row) is dict and row.get("link", False) is False
                        and proof.integrity(row.get("integrity")), "Unpinned or linked consumer dependency")
                if local_docs is not None and package == "node_modules/" + proof.DOCS:
                    require(row.get("resolved") == "file:sealed-docs.tgz", "Original Docs candidate was substituted")
                else:
                    require(public_tar_url(row.get("resolved")), "Only official public npm package origins are allowed")
                for name in proof.ORDER:
                    require(not package.endswith("node_modules/" + name) or package == "node_modules/" + name,
                            "A nested Docs package would hide the exact pair")
            evidence = []
            for name in proof.ORDER:
                row = rows[name]
                observed = packages.get("node_modules/" + name, {})
                location = observed.get("resolved")
                expected_location = location == "file:sealed-docs.tgz" if local_docs is not None and name == proof.DOCS else proof.registry_url(name, location)
                require(expected_location and observed.get("integrity") == row["integrity"]
                        and observed.get("version") == row["version"], "Exact locked Docs identity differs")
                evidence.append({**row, "resolved": location})
            lock_bytes = lock_path.read_bytes()
            self.invoke(root, environment, ["ci", *FLAGS], 360, "Clean consumer install failed")
            require(lock_path.read_bytes() == lock_bytes, "Consumer lock changed during frozen install")
            for name in proof.ORDER:
                package = root / "node_modules" / name
                require(package.is_dir() and not package.is_symlink() and not package.parent.is_symlink()
                        and package.resolve().is_relative_to(root.resolve()), "Consumer Docs must not be linked")
                manifest = checked_json(package / "package.json", root, 1024**2)
                require(manifest.get("name") == name and manifest.get("version") == proof.VERSION,
                        "Installed Docs package identity differs")
                proof.metadata_for(name, {field: manifest.get(field, {}) for field in proof.DEP_FIELDS})
            return True if local_docs is not None else {"schemaVersion": 1, "kind": "docs-public-install-v1",
                "sourceSHA": source_sha, "sourceTree": source_tree, "packages": evidence}
