"""Actual disabled/live-review entry path with wholly controlled local I/O."""
# cspell:words urllib importlib NOSYSTEM
from io import BytesIO
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import urllib.error

spec = importlib.util.spec_from_file_location("protected_writer_test", Path(__file__).with_name("docs-pair-protected-writer.py"))
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)
handoff_tests = source.module("protected_writer_fixture", "docs-pair-reviewed-handoff.test.py")
npm_tests = source.module("protected_writer_npm_fixture", "docs-pair-npm-transport.test.py")


class Reply(BytesIO):
    def getcode(self):
        return 200


class FakeWorld:
    def __init__(self, case):
        self.case = case
        self.fixture = handoff_tests.Boundary()
        self.fixture.setUp()
        self.origins = self.fixture.origins
        self.source = self.fixture.selection
        self.trace, self.journal, self.visible = [], [], set()
        self.web = source.HTTP(self)
        self.failure = self.missing = self.withdrawn = False
        self.tag_race = False
        self.transient = 0
        self.now = 0
        self.rows = {x["name"]: x for x in self.fixture.receipt["packages"]}
        self.theme, self.docs = source.ORDER
        self.src = self.source["sourceSHA"]
        self.tree = self.source["sourceTree"]
        self.state = {"GITHUB_SHA": self.src, "GITHUB_WORKFLOW_SHA": self.src,
                      "GITHUB_ACTIONS": "true", "RUNNER_ENVIRONMENT": "github-hosted",
                      "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_REPOSITORY_ID": "507542208",
                      "GITHUB_REF": "refs/heads/main", "GITHUB_REF_PROTECTED": "true",
                      "GITHUB_EVENT_NAME": "workflow_dispatch", "GITHUB_RUN_ATTEMPT": "1",
                      "GITHUB_JOB": source.JOB_ID, "GITHUB_WORKFLOW_REF": "farm-fe/farm/" + source.binding.WRITER + "@refs/heads/main",
                      "GITHUB_RUN_ID": "20", "GITHUB_ACTOR_ID": "22", "GITHUB_EVENT_PATH": "fixture", "GH_TOKEN": "fake-gh-readonly",
                      "NPM_TOKEN": "forbidden-legacy-fallback", source.TOKEN_ALIAS: "fake-only-reviewed-alias"}

    def url(self, name):
        return source.REGISTRY + name + "/-/" + name.split("/")[1] + "-2.0.0.tgz"

    def open(self, req, timeout):
        self.case.assertEqual(req.get_method(), "GET")
        u = req.full_url
        self.trace.append(("GET", u))
        def reply(obj):
            return Reply(json.dumps(obj).encode())
        def fail(code, headers=None):
            raise urllib.error.HTTPError(u, code, "fixture", headers or {}, None)
        if u.startswith(source.GITHUB + "/"):
            self.case.assertEqual(req.get_header("Authorization"), "Bearer fake-gh-readonly")
            route = u[len(source.GITHUB):]
            if route.endswith("/zip"):
                self.case.assertEqual(route, "/actions/artifacts/30/zip")
                fail(302, {"Location": "https://safe.blob.core.windows.net/object?stub=1"})
            return reply(self.origins[route])
        self.case.assertIsNone(req.get_header("Authorization"))
        if u == "https://safe.blob.core.windows.net/object?stub=1":
            return Reply(self.fixture.zip)
        for name in source.ORDER:
            url = source.PublicRegistry.endpoint(name)
            if u == url:
                tags = {"latest": "0.0.1"}
                if name in self.visible:
                    tags[source.STAGING_TAG] = "2.0.0"
                    if self.tag_race:
                        tags["latest"] = "9.0.0"
                return reply({"name": name, "dist-tags": tags})
            if u == url + "/2.0.0":
                if name == self.theme and name in self.visible and self.transient:
                    self.transient -= 1
                    fail(503)
                if name not in self.visible:
                    fail(404)
                fields = {"dependencies": {self.theme: "2.0.0"}} if name == self.docs else {
                          "peerDependencies": {self.docs: "2.0.0"}}
                return reply({"name": name, "version": "2.0.0", **fields,
                              "dist": {"integrity": self.rows[name]["integrity"], "tarball": self.url(name)}})
            if u == self.url(name):
                return Reply(self.fixture.bodies[name])
        self.case.fail("Unexpected transport URL")

    def validate_pair(self, rows, archives):
        self.case.assertEqual(rows, self.rows)
        self.case.assertEqual(archives, self.fixture.bodies)
        self.trace.append(("preflight", "both archives"))

    def publish(self, row, body, *, token_supplier):
        self.case.assertEqual(token_supplier(source.TOKEN_ALIAS), "fake-only-reviewed-alias")
        name = row["name"]
        self.case.assertEqual(self.journal[-1]["state"], "submission-pending")
        self.case.assertEqual(self.journal[-1]["attempted"][-1], name)
        self.case.assertEqual(body, self.fixture.bodies[name])
        self.trace.append(("one submit", name))
        if not self.missing:
            self.visible.add(name)
        if self.failure:
            raise ValueError("synthetic upload uncertainty")
        return True

    def verify_installed(self, rows, *, source_sha, source_tree, local_docs=None):
        self.case.assertEqual((source_sha, source_tree), (self.src, self.tree))
        if local_docs is not None:
            self.case.assertEqual(local_docs, self.fixture.bodies[self.docs])
            self.case.assertEqual(self.visible, {self.theme})
            self.trace.append(("consumer", "public theme and frozen Docs"))
            return True
        self.case.assertEqual(self.visible, {self.theme, self.docs})
        self.trace.append(("consumer", "public pair"))
        return {**self.fixture.receipt, "kind": "docs-public-install-v1",
                "packages": [{**rows[name], "resolved": self.url(name)} for name in source.ORDER]}

    def event(self, path):
        self.case.assertEqual(path, "fixture")
        return {"inputs": {"source_sha": self.src, "source_tree": self.tree, "producer_run": "10",
                           "artifact_id": "30", "artifact_sha256": self.source["artifactSha256"],
                           "tag": source.STAGING_TAG, "confirm": "first-and-only"}}

    def run(self, make_npm=None, preflight=False):
        if preflight:
            self.state["GITHUB_JOB"] = "preflight"
        argv = ["--preflight"] if preflight else ["--node", "/trusted/node", "--npm-cli", "/trusted/npm",
                                                  "--report", "/fixture/unused"]
        return source.main(argv, self.state, get_event=self.event, checkout=lambda: (self.src, self.tree), http=self.web,
            make_npm=make_npm or (lambda *_: self), record=self.journal.append, clock=lambda: self.now,
            sleep=lambda seconds: setattr(self, "now", self.now + seconds))


class Tests(unittest.TestCase):
    def test_cli_denies_non_hosted_without_any_reads_or_effects(self):
        with self.assertRaisesRegex(ValueError, "protected publisher job"):
            source.main(["--help"], {})

    def test_real_entry_orders_one_shots_midpoint_and_whole_public_pair(self):
        f = FakeWorld(self)
        result = f.run()
        self.assertEqual(result["kind"], "docs-public-retain-only-v1")
        self.assertEqual([x for x in f.trace if x[0] in ("preflight", "one submit", "consumer")],
            [("preflight", "both archives"), ("one submit", f.theme), ("consumer", "public theme and frozen Docs"),
             ("one submit", f.docs), ("consumer", "public pair")])
        self.assertFalse(f.journal[-1]["stableTagChanged"])

    def test_entrypoint_uses_real_archive_and_subprocess_adapters_with_fake_endpoints(self):
        f = FakeWorld(self)
        n = npm_tests.Contracts()
        n.setUp()
        f.fixture.bodies = n.bodies
        f.fixture.receipt["packages"] = [n.rows[name] for name in source.ORDER]
        f.fixture.freeze()
        f.rows = n.rows
        f.state[source.TOKEN_ALIAS] = "mock-value"
        submissions = []
        def fake_subprocess(command, **kwargs):
            result = n.run_npm(command, **kwargs)
            if command[2] == "publish":
                body = Path(command[3]).read_bytes()
                name = next(name for name, original in n.bodies.items() if original == body)
                self.assertEqual(f.journal[-1]["state"], "submission-pending")
                self.assertEqual(f.journal[-1]["attempted"][-1], name)
                submissions.append(name)
                f.visible.add(name)
            return result
        result = f.run(make_npm=lambda node, cli: npm_tests.transport.NpmExecution(node, cli, run=fake_subprocess))
        self.assertEqual(submissions, [f.theme, f.docs])
        self.assertEqual(n.calls, ["--version", "publish", "--version", "install", "ci", "--version", "publish",
                                   "--version", "install", "ci"])
        self.assertEqual(result["kind"], "docs-public-retain-only-v1")

    def test_wrong_runtime_or_prior_history_never_opens_npm_credentials(self):
        for key, val in (("GITHUB_RUN_ATTEMPT", "2"), ("GITHUB_REF_PROTECTED", "false"),
                         ("GITHUB_JOB", "freeze"), ("GITHUB_WORKFLOW_SHA", "c"*40)):
            f = FakeWorld(self)
            f.state[key] = val
            with self.assertRaisesRegex(ValueError, "protected publisher job"):
                f.run()
            self.assertEqual(f.trace, [])
        f = FakeWorld(self)
        f.origins[f.fixture.history]["total_count"] = 2
        with self.assertRaisesRegex(ValueError, "publisher history"):
            f.run()
        self.assertFalse(f.journal)
        self.assertFalse(any(x[0] in ("preflight", "one submit") for x in f.trace))

    def test_actual_preflight_verifies_origins_and_never_reads_npm_token(self):
        f = FakeWorld(self)
        n = npm_tests.Contracts()
        n.setUp()
        f.fixture.bodies = n.bodies
        f.fixture.receipt["packages"] = [n.rows[name] for name in source.ORDER]
        f.fixture.freeze()
        f.rows = n.rows
        class SecretDenied(dict):
            def get(self, name, *args):
                if name in (source.TOKEN_ALIAS, "NPM_TOKEN"):
                    raise AssertionError("Preflight read an npm secret")
                return super().get(name, *args)
        f.state = SecretDenied(f.state)
        result = f.run(preflight=True)
        self.assertEqual(result["status"], "exact-first-dispatch-preflight-ready")
        self.assertFalse(f.journal)
        self.assertFalse(any(x[0] == "one submit" for x in f.trace))

    def test_exact_checkout_subprocess_never_inherits_action_or_registry_credentials(self):
        world = FakeWorld(self)
        npm = npm_tests.Contracts()
        npm.setUp()
        world.fixture.bodies = npm.bodies
        world.fixture.receipt["packages"] = [npm.rows[name] for name in source.ORDER]
        world.fixture.freeze()
        world.rows = npm.rows
        world.state["GITHUB_JOB"] = "preflight"
        def checkout_process(command, *, env, **_):
            self.assertEqual(command, ["git", "rev-parse", "HEAD", "HEAD^{tree}"])
            self.assertEqual(set(env), {"PATH", "GIT_CONFIG_NOSYSTEM", "GIT_CONFIG_GLOBAL", "GIT_TERMINAL_PROMPT"})
            return SimpleNamespace(stdout=(world.src + "\n" + world.tree + "\n").encode())
        with patch.object(source.subprocess, "run", side_effect=checkout_process):
            value = source.main(["--preflight"], world.state, get_event=world.event, http=world.web)
        self.assertEqual(value["status"], "exact-first-dispatch-preflight-ready")
        self.assertFalse(world.journal)

    def test_late_competing_dispatch_after_theme_blocks_docs_upload(self):
        world = FakeWorld(self)
        original = world.publish
        def submit(row, body, *, token_supplier):
            ok = original(row, body, token_supplier=token_supplier)
            world.origins[world.fixture.history]["total_count"] = 2
            return ok
        world.publish = submit
        with self.assertRaisesRegex(ValueError, "Prior or ambiguous publisher history"):
            world.run()
        self.assertEqual([name for op, name in world.trace if op == "one submit"], [world.theme])

    def test_uncertain_submit_never_repeats_or_advances_to_docs(self):
        for unavailable in (False, True):
            f = FakeWorld(self)
            f.failure, f.missing = True, unavailable
            with self.assertRaisesRegex(ValueError, "maintainer review|do not retry"):
                f.run()
            self.assertEqual([x for x in f.trace if x[0] == "one submit"], [("one submit", f.theme)])
            if unavailable:
                self.assertEqual(f.now, 1200)
                self.assertEqual(f.journal[-1]["state"], "submission-pending")

    def test_transient_get_does_not_reissue_upload(self):
        world = FakeWorld(self)
        world.transient = 2
        world.run()
        self.assertEqual([name for op, name in world.trace if op == "one submit"], [world.theme, world.docs])
        self.assertEqual(world.now, 3)

    def test_prior_partial_or_changed_latest_path_remains_closed(self):
        f = FakeWorld(self)
        f.visible.add(f.theme)
        with self.assertRaisesRegex(ValueError, "tag already exists"):
            f.run()
        self.assertFalse(any(x[0] == "one submit" for x in f.trace))
        f = FakeWorld(self)
        f.tag_race = True
        with self.assertRaisesRegex(ValueError, "tags changed"):
            f.run()
        self.assertEqual([x for x in f.trace if x[0] == "one submit"], [("one submit", f.theme)])

    def test_real_confined_http_denies_wrong_route_and_redirect(self):
        class Bad(FakeWorld):
            def open(self, request, timeout):
                raise urllib.error.HTTPError(request.full_url, 302, "fixture",
                                             {"Location": "https://elsewhere.invalid/private"}, None)
        f = Bad(self)
        github = source.GitHub(f.web, "fake-gh-readonly")
        with self.assertRaisesRegex(ValueError, "GitHub read route"):
            github.get("/repos/another/project")
        with self.assertRaisesRegex(ValueError, "storage redirect"):
            github.download(30, source.binding.MAX_ZIP)
        with self.assertRaisesRegex(ValueError, "HTTP 302"):
            source.PublicRegistry(f.web).tags(f.theme)


if __name__ == "__main__":
    unittest.main()
