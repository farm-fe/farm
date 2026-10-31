"""Read public registry bytes independently; never copy checksum claims from a ledger."""
# cspell:words urllib npmjs
import base64
import hashlib
import json
import os
import re
import time
import urllib.parse
import urllib.request


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, new_url):
        return None


class RegistryReadOnly:
    def __init__(self, environment=None, opener=None, clock=time.monotonic):
        environment = os.environ if environment is None else environment
        if environment.get("GITHUB_ACTIONS") != "true":
            raise PermissionError("Independent registry verification runs only in Actions")
        self.opener = opener or urllib.request.build_opener(NoRedirect())
        self.clock = clock
        self.deadline = clock() + 600
        self.total_bytes = 0
        self.verifiedArchives = {}

    def open(self, url):
        parsed = urllib.parse.urlsplit(url)
        if parsed.scheme != "https" or parsed.hostname not in ["registry.npmjs.org", "crates.io", "static.crates.io"]:
            raise ValueError("Untrusted registry origin")
        if parsed.username or parsed.password or parsed.port not in [None, 443] or parsed.fragment or parsed.query:
            raise ValueError("Unexpected registry URL fields")
        remaining = self.deadline - self.clock()
        if remaining <= 0:
            raise ValueError("Registry verification total deadline exceeded")
        # No authentication headers and no redirects to an unverified origin.
        return self.opener.open(urllib.request.Request(url, headers={"Accept": "application/json", "User-Agent": "Farm-release-read-only-verifier"}, method="GET"), timeout=min(30, remaining))

    def json(self, url):
        with self.open(url) as response:
            body = response.read(2 * 1024 * 1024 + 1)
        if len(body) > 2 * 1024 * 1024:
            raise ValueError("Registry metadata exceeded bound")
        return json.loads(body.decode("utf-8"))

    def archive(self, url):
        sha256, sha512, sha1, count = hashlib.sha256(), hashlib.sha512(), hashlib.sha1(), 0
        with self.open(url) as response:
            while True:
                if self.clock() >= self.deadline:
                    raise ValueError("Registry verification total deadline exceeded")
                body = response.read(1024 * 1024)
                if not body:
                    break
                count += len(body); self.total_bytes += len(body)
                if count > 100 * 1024 * 1024 or self.total_bytes > 3 * 1024 ** 3:
                    raise ValueError("Registry archive byte budget exceeded")
                sha256.update(body); sha512.update(body); sha1.update(body)
        if count <= 0:
            raise ValueError("Registry archive is empty")
        return {"sha256": sha256.hexdigest(), "sha512": base64.b64encode(sha512.digest()).decode("ascii"), "sha1": sha1.hexdigest()}

    def verify(self, records):
        if not isinstance(records, dict) or not records or len(records) > 1024:
            raise ValueError("Registry package inventory missing or exceeds bound")
        result = {}
        self.verifiedArchives = {}
        for identity, record in sorted(records.items()):
            kind, name = identity.split(":", 1)
            version = record.get("version")
            if not re.fullmatch(r"\d+\.\d+\.\d+", version or ""):
                raise ValueError("Exact stable registry version required")
            if kind == "npm":
                if not re.fullmatch(r"(?:@[a-z0-9._-]+/)?[a-z0-9._-]+", name):
                    raise ValueError("Unsafe npm identity")
                metadata = self.json("https://registry.npmjs.org/" + urllib.parse.quote(name, safe="") + "/" + version)
                if metadata.get("name") != name or metadata.get("version") != version:
                    raise ValueError("Public npm metadata identity differs")
                dist = metadata["dist"]; url = dist["tarball"]
                if urllib.parse.urlsplit(url).hostname != "registry.npmjs.org":
                    raise ValueError("npm archive origin differs")
                actual = self.archive(url)
                integrity = dist.get("integrity", "").split()
                if integrity:
                    if "sha512-" + actual["sha512"] not in integrity:
                        raise ValueError("Public npm archive integrity differs")
                elif dist.get("shasum") != actual["sha1"]:
                    raise ValueError("Public npm archive checksum missing or differs")
            elif kind == "cargo":
                if not re.fullmatch(r"[A-Za-z0-9_-]+", name):
                    raise ValueError("Unsafe Cargo identity")
                metadata = self.json("https://crates.io/api/v1/crates/" + name + "/" + version)["version"]
                if metadata.get("crate") != name or metadata.get("num") != version:
                    raise ValueError("Public Cargo metadata identity differs")
                url="https://static.crates.io/crates/" + name + "/" + name + "-" + version + ".crate"
                actual = self.archive(url)
                if metadata.get("checksum") != actual["sha256"]:
                    raise ValueError("Public Cargo archive checksum differs")
            else:
                raise ValueError("Unknown registry kind")
            if actual["sha256"] != record.get("archiveSha256"):
                raise ValueError("Published archive differs from successful release record")
            result[identity] = actual["sha256"]
            self.verifiedArchives[identity] = {**actual,'tarball':url}
        return result
