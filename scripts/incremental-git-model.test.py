"""Small real Git repositories verify commit/tree/rename/delete/executable-mode handling."""
# cspell:words importlib NOSYSTEM autocrlf filemode gpgsign
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("git_model_fixture", Path(__file__).with_name("incremental-chain-plan.test.py"))
fixtures = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixtures)
chain = fixtures.chain


def isolated_git_environment(source):
    # Inherited repository, object, index and configuration redirection must
    # never make a temporary-repository contract touch the caller's checkout.
    result = {key: value for key, value in source.items() if not key.upper().startswith("GIT_")}
    result.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_TERMINAL_PROMPT": "0"})
    return result


class RealGitModel(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="farm-impact-contract-"))
        self.environment = isolated_git_environment(os.environ)
        self.git("init", "--initial-branch=main")
        top = Path(self.git("rev-parse", "--show-toplevel").decode("utf-8").strip()).resolve()
        self.assertEqual(top, self.directory.resolve())
        self.git("config", "core.autocrlf", "false")
        self.git("config", "core.filemode", "true")
        files, self.family, _ = fixtures.fixture()
        files["docs/releases/v2-candidate.json"] = json.dumps(self.family).encode()
        for name, body in files.items():
            file = self.directory / name; file.parent.mkdir(parents=True, exist_ok=True); file.write_bytes(body)
        self.git("add", "."); self.commit("base")
        self.base = self.snapshot()

    def git(self, *args):
        command = ["git", "-c", "core.hooksPath=" + str(self.directory / "empty-hooks"),
                   "-c", "commit.gpgsign=false", "-c", "user.name=Farm CI Contract", "-c", "user.email=ci@example.invalid", *args]
        trace = os.environ.get("FARM_INCREMENTAL_GIT_TRACE")
        if trace:
            with Path(trace).open("a", encoding="utf-8") as output:
                output.write(json.dumps({"cwd": str(self.directory), "command": command}) + "\n")
        return subprocess.run(command, cwd=self.directory, env=self.environment, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, timeout=20, check=True).stdout

    def test_hostile_git_environment_cannot_redirect_the_temporary_repository(self):
        hostile = {**os.environ, "GIT_DIR": str(Path.cwd() / ".git"), "GIT_WORK_TREE": str(Path.cwd()),
                   "GIT_INDEX_FILE": str(Path.cwd() / "DO-NOT-WRITE"), "GIT_OBJECT_DIRECTORY": "wrong-objects",
                   "GIT_CONFIG_COUNT": "1", "GIT_CONFIG_KEY_0": "core.worktree", "GIT_CONFIG_VALUE_0": str(Path.cwd()),
                   "git_common_dir": "wrong-common", "GIT_ALTERNATE_OBJECT_DIRECTORIES": "wrong-alternates"}
        previous = self.environment
        try:
            self.environment = isolated_git_environment(hostile)
            allowed = {"GIT_CONFIG_NOSYSTEM", "GIT_CONFIG_GLOBAL", "GIT_TERMINAL_PROMPT"}
            self.assertEqual({key for key in self.environment if key.upper().startswith("GIT_")}, allowed)
            actual = self.git("rev-parse", "--show-toplevel").decode("utf-8").strip()
            self.assertEqual(Path(actual).resolve(), self.directory.resolve())
            self.assertEqual(self.git("rev-parse", "--git-path", "index").decode().strip(), ".git/index")
        finally:
            self.environment = previous

    def commit(self, message):
        self.git("commit", "-m", message)

    def snapshot(self):
        sha = self.git("rev-parse", "HEAD").decode().strip()
        tree = self.git("rev-parse", "HEAD^{tree}").decode().strip()
        files, modes = {}, {}
        for row in self.git("ls-tree", "-rz", "HEAD").split(b"\0"):
            if not row:
                continue
            meta, name = row.split(b"\t", 1); mode, kind, oid = meta.decode().split(); self.assertEqual(kind, "blob")
            file = name.decode("utf-8"); files[file] = self.git("cat-file", "blob", oid); modes[file] = mode
        return {"sha": sha, "tree": tree}, files, modes

    def select(self):
        source, files, modes = self.snapshot(); original, old_files, old_modes = self.base
        ledger, anchor = fixtures.baseline(old_files, self.family, "ci")
        ledger.update({"sourceSHA": original["sha"], "sourceTree": original["tree"],
                       "sourceSnapshotHash": chain.digest(chain.snapshot_records(old_files, old_modes))})
        anchor.update({"sourceSHA": original["sha"], "sourceTree": original["tree"], "ledgerSha256": chain.digest(ledger)})
        with mock.patch.object(chain.impact, "git", self.git):
            changes = chain.impact.changed_paths(original["sha"], source["sha"])
        return chain.plan_chain("ci", source, files, self.family, changes, ledger, old_files, anchor,
                                source_modes=modes, baseline_modes=old_modes), changes

    def test_real_executable_mode_change_preserves_bytes_but_changes_selection(self):
        self.git("update-index", "--chmod=+x", "packages/a/src/index.ts"); self.commit("mode change")
        source, files, modes = self.snapshot()
        self.assertEqual(files["packages/a/src/index.ts"], self.base[1]["packages/a/src/index.ts"])
        self.assertEqual(modes["packages/a/src/index.ts"], "100755")
        plan, changes = self.select()
        self.assertEqual(set(plan["build"]), {"npm:a", "npm:b"})
        self.assertEqual(changes, [{"status": "M", "paths": ["packages/a/src/index.ts"]}])

    def test_real_cross_owner_rename_has_two_paths(self):
        self.git("mv", "packages/a/src/index.ts", "packages/b/src/moved.ts"); self.commit("rename")
        plan, changes = self.select()
        self.assertEqual(set(plan["build"]), {"npm:a", "npm:b"})
        self.assertEqual(changes, [{"status": "R100", "paths": ["packages/a/src/index.ts", "packages/b/src/moved.ts"]}])

    def test_real_deletion_and_commit_tree_types_remain_distinct(self):
        self.git("rm", "packages/a/src/index.ts"); self.commit("delete")
        plan, changes = self.select(); source = self.snapshot()[0]
        self.assertEqual(set(plan["build"]), {"npm:a", "npm:b"})
        self.assertEqual(changes, [{"status": "D", "paths": ["packages/a/src/index.ts"]}])
        self.assertEqual(chain.impact.revision(source["sha"], self.git), source["sha"])
        self.assertEqual(chain.impact.tree_revision(source["tree"], self.git), source["tree"])
        with self.assertRaises(subprocess.CalledProcessError):
            chain.impact.revision(source["tree"], self.git)


if __name__ == "__main__":
    unittest.main(verbosity=2)
