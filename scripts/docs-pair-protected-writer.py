"""Controlled exact-source runner for the two reviewed Docs archives.

Only a maintainer's first-attempt manual dispatch on protected main can enter
this writer. A registry 404 never authorizes a repeat after a possible write.
"""
# cspell:words importlib urllib argparse npmjs ITIMER SIGALRM setitimer getitimer fsync fdopen O_NOFOLLOW WRONLY CREAT NOSYSTEM readback
import argparse
import base64
from contextlib import contextmanager
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


binding = module("docs_protected_binding", "docs-pair-reviewed-handoff.py")
proof = module("docs_protected_registry_proof", "docs-pair-registry-proof.py")
native = module("docs_protected_http", "ci-native-evidence.py")
require = binding.require
TOKEN_ALIAS = "NPM_BYPASS_2FA_TOKEN"
STAGING_TAG = "farm-docs-bootstrap"
JOB_ID = "publish-pair"
REGISTRY = "https://registry.npmjs.org/"
GITHUB = "https://api.github.com/repos/farm-fe/farm"
ORDER = proof.ORDER
INPUTS = {"source_sha", "source_tree", "producer_run", "artifact_id", "artifact_sha256", "tag", "confirm"}
ROUTE = re.compile(r"/(?:branches/main|git/commits/[0-9a-f]{40}|actions/(?:artifacts/[1-9][0-9]{0,19}"
                   r"|runs/[1-9][0-9]{0,19}(?:/attempts/1(?:/jobs)?)?"
                   r"|workflows/(?:docs-pair-publish\.yaml|[1-9][0-9]{0,19}/runs\?branch=main&event=workflow_dispatch&per_page=2)))")


def positive_field(value):
    require(isinstance(value, str) and re.fullmatch(r"[1-9][0-9]{0,19}", value), "Invalid fixed dispatch identifier")
    return int(value)


def context(env, *, read_event, checkout, preflight=False):
    sha = env.get("GITHUB_SHA")
    require(binding.sha(sha, 40) and env.get("GITHUB_WORKFLOW_SHA") == sha
            and env.get("GITHUB_ACTIONS") == "true" and env.get("RUNNER_ENVIRONMENT") == "github-hosted"
            and env.get("GITHUB_REPOSITORY") == "farm-fe/farm"
            and env.get("GITHUB_REPOSITORY_ID") == str(binding.REPOSITORY_ID)
            and env.get("GITHUB_REF") == "refs/heads/main" and env.get("GITHUB_REF_PROTECTED") == "true"
            and env.get("GITHUB_EVENT_NAME") == "workflow_dispatch" and env.get("GITHUB_RUN_ATTEMPT") == "1"
            and env.get("GITHUB_JOB") == ("preflight" if preflight else JOB_ID)
            and env.get("GITHUB_WORKFLOW_REF") == "farm-fe/farm/" + binding.WRITER + "@refs/heads/main",
            "Exact hosted protected publisher job and first attempt required")
    writer = positive_field(env.get("GITHUB_RUN_ID"))
    actor = positive_field(env.get("GITHUB_ACTOR_ID"))
    event = read_event(env["GITHUB_EVENT_PATH"])
    inputs = event.get("inputs", {})
    require(type(inputs) is dict and set(inputs) == INPUTS and inputs["confirm"] == "first-and-only"
            and inputs["source_sha"] == sha and inputs["tag"] == STAGING_TAG
            and binding.sha(inputs["source_tree"], 40)
            and binding.sha(inputs["artifact_sha256"], 64), "Exact reviewed Docs writer inputs required")
    require(checkout() == (sha, inputs["source_tree"]), "Writer worktree differs from reviewed source and tree")
    return {"sourceSHA": sha, "sourceTree": inputs["source_tree"],
            "producerRun": positive_field(inputs["producer_run"]), "writerRun": writer,
            "artifactId": positive_field(inputs["artifact_id"]), "artifactSha256": inputs["artifact_sha256"],
            "tag": inputs["tag"]}, actor


def read_json_bytes(raw):
    return binding.receipt_bytes(raw)


@contextmanager
def read_deadline(seconds):
    # Socket idle timeouts alone do not bound a slow byte stream.
    require(0 < seconds <= 20 and signal.getitimer(signal.ITIMER_REAL)[0] == 0,
            "Invalid or already occupied protected read budget")
    previous = signal.getsignal(signal.SIGALRM)
    def expire(*_):
        raise TimeoutError("Protected read deadline expired")
    signal.signal(signal.SIGALRM, expire)
    signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)


class TransientRead(ValueError):
    pass


class HTTP:
    def __init__(self, opener=None):
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), native.NoRedirect()) if opener is None else opener

    def request(self, url, *, limit, headers=None, missing=False, redirect=False, timeout=20):
        try:
            with read_deadline(timeout):
                return self._read(url, limit, headers, missing, redirect, timeout)
        except TimeoutError:
            raise TransientRead("Protected read deadline expired") from None

    def _read(self, url, limit, headers, missing, redirect, timeout):
        req = urllib.request.Request(url, headers={**(headers or {}), "User-Agent": "farm-docs-release"}, method="GET")
        try:
            with self.opener.open(req, timeout=timeout) as response:
                require(response.getcode() == 200, "Unexpected read response")
                raw = response.read(limit + 1)
                require(0 < len(raw) <= limit, "Unexpected read size")
                return raw
        except urllib.error.HTTPError as error:
            if error.code == 404 and missing:
                return None
            if error.code == 302 and redirect:
                location = native.artifact_redirect_url(error.headers.get("Location", ""))
                return self._read(location, limit, None, False, False, timeout)
            if error.code in (408, 429) or 500 <= error.code <= 599:
                raise TransientRead("Protected read transient HTTP " + str(error.code)) from None
            raise ValueError("Protected read rejected: HTTP " + str(error.code)) from None
        except (urllib.error.URLError, OSError):
            raise TransientRead("Protected read unavailable") from None


class GitHub:
    def __init__(self, http, token):
        require(isinstance(token, str) and 0 < len(token) <= 4096 and not any(c in token for c in "\0\r\n"),
                "Existing scoped Actions read identity required")
        self.http = http
        self.header = {"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
                       "X-GitHub-Api-Version": "2022-11-28"}

    def get(self, route):
        require(isinstance(route, str) and ROUTE.fullmatch(route), "GitHub read route denied")
        return read_json_bytes(self.http.request(GITHUB + route, limit=2 * 1024**2, headers=self.header))

    def download(self, artifact, limit):
        require(binding.positive_id(artifact) and limit == binding.MAX_ZIP, "Exact frozen artifact and bound required")
        return self.http.request(f"{GITHUB}/actions/artifacts/{artifact}/zip", limit=limit,
                                 headers=self.header, redirect=True)


class PublicRegistry:
    def __init__(self, http):
        self.http = http

    @staticmethod
    def endpoint(name):
        require(name in ORDER, "Only the reviewed two Docs names may enter registry transport")
        return REGISTRY + urllib.parse.quote(name, safe="")

    def tags(self, name, timeout=20):
        raw = self.http.request(self.endpoint(name), limit=2 * 1024**2,
                                headers={"Accept": "application/vnd.npm.install-v1+json", "Cache-Control": "no-cache"}, timeout=timeout)
        record = read_json_bytes(raw)
        tags = record.get("dist-tags")
        require(record.get("name") == name and type(tags) is dict
                and all(isinstance(k, str) and isinstance(v, str) for k, v in tags.items()),
                "Cannot establish immutable pre-existing package tag state")
        return tags

    def lookup(self, row, *, verify_archive=False, timeout=20):
        name = row.get("name")
        require(row.get("kind") == "npm" and row.get("version") == proof.VERSION, "Only exact Docs 2.0.0 reads admitted")
        deadline = time.monotonic() + timeout
        raw = self.http.request(self.endpoint(name) + "/" + proof.VERSION, limit=2 * 1024**2, headers={"Cache-Control": "no-cache"}, missing=True, timeout=timeout)
        if raw is None:
            return {"name": name, "version": proof.VERSION, "status": "missing"}
        info = read_json_bytes(raw)
        require(info.get("name") == name and info.get("version") == proof.VERSION, "Public package identity differs")
        dist = info.get("dist", {})
        url, sri = dist.get("tarball"), dist.get("integrity")
        require(proof.registry_url(name, url) and proof.integrity(sri), "Public original archive URL or integrity differs")
        metadata = {field: info.get(field, {}) for field in proof.DEP_FIELDS}
        proof.metadata_for(name, metadata)
        result = {"name": name, "version": proof.VERSION, "status": "published", "integrity": sri,
                  "tarballURL": url, "metadata": metadata, "evidenceKind": "registry-metadata"}
        if verify_archive:
            body = self.http.request(url, limit=binding.MAX_TAR, timeout=deadline - time.monotonic())
            actual = "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode("ascii")
            require(actual == sri, "Public exact archive integrity differs")
            result.update(archiveSha256=hashlib.sha256(body).hexdigest(), archiveBytes=len(body),
                          evidenceKind="registry-archive")
        return result


def exact_public(registry, expected, timeout):
    looked = registry.lookup({"kind": "npm", **expected}, verify_archive=True, timeout=timeout)
    if looked["status"] == "missing":
        return False
    require(looked["status"] == "published" and all(looked[field] == expected[field] for field in
            ("name", "version", "integrity", "archiveSha256", "archiveBytes")), "Public bytes do not match original")
    return True


def prepare(selection, actor_id, *, github, registry, validate):
    resolved = binding.read_reviewed_handoff(selection, actor_id, get=github.get, download=github.download)
    frozen = resolved["receipt"]
    rows = {row["name"]: row for row in frozen["packages"]}
    saved_tags = {name: registry.tags(name) for name in ORDER}
    for name in ORDER:
        require(STAGING_TAG not in saved_tags[name], "Proposed staging tag already exists")
        require(not exact_public(registry, rows[name], 20), "Existing Docs 2.0.0 requires read-only reconciliation")
    # Ensure both original archives and the clean npm subprocess are usable
    # before resolving the single configured npm token.
    validate(rows, resolved["archives"])
    return resolved, frozen, rows, saved_tags


def write_once(selection, actor_id, *, github, registry, npm, token, record, clock=time.monotonic, sleep=time.sleep):
    resolved, frozen, rows, saved_tags = prepare(selection, actor_id,
        github=github, registry=registry, validate=npm.validate_pair)
    attempted, public = [], []
    state = {"artifactId": resolved["artifactId"], "sourceSHA": selection["sourceSHA"], "sourceTree": selection["sourceTree"],
             "writerRun": selection["writerRun"], "attempt": 1, "attempted": attempted, "publicVerified": public,
             "stableTagChanged": False, "state": "first-dispatch-no-write"}
    def checkpoint(status):
        state["state"] = status
        record(json.loads(json.dumps(state)))

    checkpoint("first-dispatch-no-write")
    for name in ORDER:
        if name == proof.DOCS:
            require(npm.verify_installed(rows, source_sha=selection["sourceSHA"], source_tree=selection["sourceTree"],
                                         local_docs=resolved["archives"][proof.DOCS]) is True,
                    "Public theme is not consumable with the frozen Docs peer")
        binding.require_main(github.get, selection["sourceSHA"], selection["sourceTree"])
        binding.require_first_writer(github.get, selection["writerRun"], resolved["writerWorkflowId"])
        require(registry.tags(name) == saved_tags[name] and not exact_public(registry, rows[name], 20),
                "Registry changed after first dispatch; do not submit")
        attempted.append(name)
        checkpoint("submission-pending")
        accepted = True
        try:
            accepted = npm.publish(rows[name], resolved["archives"][name], token_supplier=token) is True
        except Exception:
            accepted = False
        deadline, delay = clock() + 1200, 1
        while True:
            now = clock()
            require(now < deadline, "Upload outcome unresolved; do not retry or continue")
            try:
                if name not in public and exact_public(registry, rows[name], min(20, max(1, int(deadline - now)))):
                    public.append(name)
                    checkpoint("public-byte-verified")
                if name in public:
                    if not accepted:
                        raise ValueError("Ambiguous submission reconciled by read only; stop for maintainer review")
                    fresh = registry.tags(name, timeout=min(20, deadline - clock()))
                    require(fresh in (saved_tags[name], {**saved_tags[name], STAGING_TAG: proof.VERSION}),
                            "Public stable/other tags changed; stop before any further write")
                    if fresh.get(STAGING_TAG) == proof.VERSION:
                        break
            except TransientRead:
                pass  # Retry only verification reads, never the upload.
            sleep(min(delay, max(0, deadline - clock())))
            delay = min(delay * 2, 30)
    installed = npm.verify_installed(rows, source_sha=selection["sourceSHA"], source_tree=selection["sourceTree"])
    retained = proof.verify_pair(frozen, installed, expected_source_sha=selection["sourceSHA"],
                                 expected_source_tree=selection["sourceTree"], lookup=registry.lookup)
    require(all(registry.tags(name) == {**saved_tags[name], STAGING_TAG: proof.VERSION} for name in ORDER),
            "Final tag readback differs")
    checkpoint("exact-pair-public-signed-adoption-pending")
    return retained


def main(argv=None, env=None, *, get_event=None, checkout=None, http=None,
         make_npm=None, record=None, clock=time.monotonic, sleep=time.sleep):
    env = os.environ if env is None else env
    def real_event(file):
        with open(file, "rb") as stream:
            raw = stream.read(65537)
        require(0 < len(raw) <= 65536, "Fixed dispatch event size differs")
        return read_json_bytes(raw)
    def real_checkout():
        value = subprocess.run(["git", "rev-parse", "HEAD", "HEAD^{tree}"], check=True,
            cwd=Path(__file__).resolve().parents[1], capture_output=True, timeout=10,
            env={"PATH": os.defpath, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
                 "GIT_TERMINAL_PROMPT": "0"})
        return tuple(value.stdout.decode("ascii").splitlines())
    argv = sys.argv[1:] if argv is None else argv
    preflight = argv == ["--preflight"]
    selection, actor_id = context(env, read_event=get_event or real_event, checkout=checkout or real_checkout,
                                  preflight=preflight)
    if preflight:
        http = HTTP() if http is None else http
        github = GitHub(http, env.get("GH_TOKEN"))
        transport = module("preflight_docs_npm", "docs-pair-npm-transport.py")
        prepare(selection, actor_id, github=github, registry=PublicRegistry(http), validate=transport.validate_pair)
        return {"status": "exact-first-dispatch-preflight-ready", "artifactId": selection["artifactId"],
                "sourceSHA": selection["sourceSHA"], "sourceTree": selection["sourceTree"]}
    parser = argparse.ArgumentParser()
    parser.add_argument("--node", required=True)
    parser.add_argument("--npm-cli", required=True)
    parser.add_argument("--report", required=True)
    args = parser.parse_args(argv)
    if record is None:
        runner_temp = Path(env["RUNNER_TEMP"]).resolve(strict=True)
        path = Path(args.report)
        require(path.parent.resolve(strict=True) == runner_temp and path.name == "docs-pair-publish-journal.jsonl",
                "Exact isolated runner journal path required")
        stream = None
        def record(report):
            nonlocal stream
            if stream is None:
                stream = os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600),
                                   "w", encoding="utf-8")
            stream.write(json.dumps(report, sort_keys=True) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
    else:
        stream = None
    if make_npm is None:
        make_npm = module("docs_protected_npm", "docs-pair-npm-transport.py").NpmExecution
    http = HTTP() if http is None else http
    github = GitHub(http, env.get("GH_TOKEN"))
    try:
        retained = write_once(selection, actor_id, github=github, registry=PublicRegistry(http),
            npm=make_npm(Path(args.node), Path(args.npm_cli)),
            token=lambda alias: env.get(TOKEN_ALIAS) if alias == TOKEN_ALIAS else None, record=record,
            clock=clock, sleep=sleep)
        if stream is not None:
            evidence = path.with_name("docs-pair-retained.json")
            with os.fdopen(os.open(evidence, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600),
                           "w", encoding="utf-8") as output:
                output.write(json.dumps(retained, sort_keys=True) + "\n")
                output.flush()
                os.fsync(output.fileno())
        return retained
    finally:
        if stream is not None:
            stream.close()


if __name__ == "__main__":
    try:
        result = main()
        if result.get("status") == "exact-first-dispatch-preflight-ready":
            print(json.dumps(result, sort_keys=True))
    except Exception:
        raise SystemExit("Protected Docs publication stopped; inspect sanitized journal, never retry automatically") from None
