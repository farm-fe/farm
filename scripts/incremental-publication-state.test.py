"""Low-cost publication state contracts; no registry writes or credentials."""
# cspell:words importlib
import copy
import base64
import json
import hashlib
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("publication", Path(__file__).with_name("incremental-publication-state.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class GitAPI:
    def __init__(self):
        self.tip, self.commits, self.counter = None, {}, 0

    def read_ref(self, reference):
        return self.tip

    def read_state(self, commit):
        return copy.deepcopy(self.commits[commit]["state"])

    def create_state_commit(self, state, parents):
        self.counter += 1
        commit = hashlib.sha1(json.dumps({"state": state, "parents": parents}, sort_keys=True).encode()).hexdigest()
        self.commits[commit] = {"state": copy.deepcopy(state), "parents": parents}
        return commit

    def create_ref(self, reference, commit):
        if self.tip is not None:
            raise ValueError("Concurrent ref creation")
        self.tip = commit

    def update_ref(self, reference, commit, force):
        assert force is False
        pending, ancestors = [commit], set()
        while pending:
            current = pending.pop()
            if current in ancestors:
                continue
            ancestors.add(current); pending.extend(self.commits[current]["parents"])
        if self.tip not in ancestors:
            raise ValueError("Non fast forward publication state")
        self.tip = commit


class Registry:
    def __init__(self):
        self.values, self.writes, self.crash_after = {}, [], None

    def lookup(self, row):
        return self.values.get(row["name"])

    def publish(self, row, body):
        self.writes.append(row["name"])
        self.values[row["name"]] = hashlib.sha256(body).hexdigest()
        if self.crash_after == row["name"]:
            raise RuntimeError("Simulated crash after registry accepted bytes")


def fixture():
    bodies = {"sdk": b"original Rust archive", "core": b"original npm archive"}
    rows = [{"kind": kind, "name": name, "version": "2.0.0", "archiveBytes": len(bodies[name]),
             "archiveSha256": hashlib.sha256(bodies[name]).hexdigest()} for kind, name in [("npm", "core"), ("cargo", "sdk")]]
    manifest = {"chain": "release", "profile": "release", "sourceSHA": "a" * 40, "sourceTree": "b" * 40,
                "planHash": "c" * 64, "baseline": None, "packages": rows}
    api = GitAPI()
    return api, module.GitRefStore(api), manifest, bodies, Registry()


class Contracts(unittest.TestCase):
    def runner(self, store, manifest, attempt=1, authorize=lambda value: True):
        return module.Publication(store, manifest, {"runId": 7, "attempt": attempt}, authorize)

    def proof(self, manifest):
        return {"manifestHash": module.frozen_manifest(manifest), "verified": True}

    def finish(self, manifest, proof):
        return {"manifestHash": module.frozen_manifest(manifest), "sourceSHA": manifest["sourceSHA"], "verified": True}

    def test_success_advances_baseline_only_after_acceptance_and_cargo_first(self):
        api, store, manifest, bodies, registry = fixture()
        runner = self.runner(store, manifest); transaction = runner.acquire()
        def accept(value):
            self.assertIsNone(store.read()[1]["baseline"])
            self.assertEqual(store.read()[1]["active"]["phase"], "public-acceptance")
            return self.proof(value)
        self.assertEqual(runner.execute(lambda row: bodies[row["name"]], registry, accept, self.finish), transaction)
        self.assertEqual(registry.writes, ["sdk", "core"])
        state = store.read()[1]
        self.assertEqual(state["baseline"], transaction); self.assertIsNone(state["active"])

    def test_active_owner_cannot_be_stolen_or_reentered(self):
        api, store, manifest, bodies, registry = fixture()
        self.runner(store, manifest).acquire()
        for attempt in [1, 2]:
            with self.assertRaisesRegex(ValueError, "still active"):
                self.runner(store, manifest, attempt).acquire()

    def test_same_run_new_instance_cannot_publish_without_owned_fence(self):
        api, store, manifest, bodies, registry = fixture()
        self.runner(store, manifest).acquire()
        with self.assertRaisesRegex(ValueError, "ownership lost"):
            self.runner(store, manifest).upload(lambda row: bodies[row["name"]], registry)
        self.assertEqual(registry.writes, [])

    def test_explicit_handoff_claim_is_exclusive_and_invalidates_old_holder(self):
        api, store, manifest, bodies, registry = fixture()
        first = self.runner(store, manifest); first.acquire()
        first.upload(lambda row: bodies[row["name"]], registry)
        first.handoff("public-acceptance")
        second = self.runner(store, manifest); second.claim_handoff("public-acceptance")
        with self.assertRaisesRegex(ValueError, "handoff missing"):
            self.runner(store, manifest).claim_handoff("public-acceptance")
        with self.assertRaisesRegex(ValueError, "ownership lost"):
            first.owned()
        second.finalize(self.proof, self.finish)

    def test_late_npm_collision_prevents_any_rust_upload(self):
        api, store, manifest, bodies, registry = fixture()
        registry.values["core"] = "f" * 64
        runner = self.runner(store, manifest); runner.acquire()
        with self.assertRaisesRegex(ValueError, "different bytes"):
            runner.upload(lambda row: bodies[row["name"]], registry)
        self.assertEqual(registry.writes, [])

    def test_release_failure_retains_lock_and_old_success_baseline(self):
        api, store, manifest, bodies, registry = fixture()
        runner = self.runner(store, manifest); runner.acquire()
        def fail(value, proof):
            raise RuntimeError("Simulated Release write failure")
        with self.assertRaises(RuntimeError):
            runner.execute(lambda row: bodies[row["name"]], registry, self.proof, fail)
        state = store.read()[1]
        self.assertIsNone(state["baseline"])
        self.assertEqual(state["active"]["phase"], "release-finalization")
        runner.finalize(self.proof, self.finish)
        self.assertIsNone(store.read()[1]["active"])

    def test_crash_after_upload_resumes_original_bytes_without_duplicate_write(self):
        api, store, manifest, bodies, registry = fixture()
        first = self.runner(store, manifest); first.acquire(); registry.crash_after = "sdk"
        with self.assertRaises(RuntimeError):
            first.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        self.assertIsNone(store.read()[1]["baseline"])
        self.assertEqual(store.read()[1]["active"]["packages"]["cargo:sdk"], "upload-started")
        registry.crash_after = None
        second = self.runner(store, manifest, 2); second.acquire(lambda owner: owner == {"runId": 7, "attempt": 1})
        second.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        self.assertEqual(registry.writes, ["sdk", "core"])
        with self.assertRaisesRegex(ValueError, "ownership lost"):
            first.owned()

    def test_recovery_rejects_changed_source_plan_version_or_archive(self):
        for key in ["sourceSHA", "planHash", "packages"]:
            api, store, manifest, bodies, registry = fixture()
            self.runner(store, manifest).acquire()
            changed = copy.deepcopy(manifest)
            if key == "packages":
                changed[key][0]["archiveSha256"] = "f" * 64
            else:
                changed[key] = "f" * len(changed[key])
            with self.assertRaisesRegex(ValueError, "original manifest"):
                self.runner(store, changed, 2).acquire(lambda owner: True)

    def test_rebuilt_bytes_refused_before_any_upload(self):
        api, store, manifest, bodies, registry = fixture()
        runner = self.runner(store, manifest); runner.acquire()
        bodies["core"] = b"rebuilt bytes"
        with self.assertRaisesRegex(ValueError, "archive differs"):
            runner.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        self.assertEqual(registry.writes, [])

    def test_same_version_other_bytes_cannot_be_reused(self):
        api, store, manifest, bodies, registry = fixture()
        registry.values["sdk"] = "f" * 64
        runner = self.runner(store, manifest); runner.acquire()
        with self.assertRaisesRegex(ValueError, "different bytes"):
            runner.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        self.assertEqual(registry.writes, [])

    def test_failed_acceptance_retains_lock_and_does_not_advance_baseline(self):
        api, store, manifest, bodies, registry = fixture()
        runner = self.runner(store, manifest); runner.acquire()
        with self.assertRaisesRegex(ValueError, "acceptance missing"):
            runner.execute(lambda row: bodies[row["name"]], registry, lambda value: {"verified": False}, self.finish)
        self.assertIsNone(store.read()[1]["baseline"])
        self.assertIsNotNone(store.read()[1]["active"])

    def test_stale_baseline_requires_new_plan(self):
        api, store, manifest, bodies, registry = fixture()
        store.swap(None, {"baseline": "d" * 64, "active": None})
        with self.assertRaisesRegex(ValueError, "baseline moved"):
            self.runner(store, manifest).acquire()

    def test_parallel_first_acquisition_has_only_one_winner(self):
        api, store, manifest, bodies, registry = fixture()
        left_revision, left = store.read(); right_revision, right = store.read()
        store.swap(left_revision, left)
        with self.assertRaisesRegex(ValueError, "Concurrent ref"):
            store.swap(right_revision, right)

    def test_parallel_updates_reject_sibling_commit_without_retry(self):
        api, store, manifest, bodies, registry = fixture()
        store.swap(None, {"baseline": None, "active": None})
        left_revision, left = store.read(); right_revision, right = store.read()
        store.swap(left_revision, left)
        with self.assertRaisesRegex(ValueError, "Non fast forward"):
            store.swap(right_revision, right)

    def test_content_addressed_equal_proposals_are_distinct_store_operations(self):
        api, store, manifest, bodies, registry = fixture()
        state = {"baseline": None, "active": None}
        # Git itself gives identical content the same identity.
        self.assertEqual(api.create_state_commit(state, []), api.create_state_commit(state, []))
        first = store.swap(None, state)
        with self.assertRaises(ValueError):
            store.swap(None, state)
        self.assertNotEqual(list(api.commits)[-1], first)

    def test_final_commit_response_loss_resumes_as_completed_without_upload(self):
        api, store, manifest, bodies, registry = fixture()
        original = api.update_ref
        def response_lost(reference, commit, force):
            original(reference, commit, force)
            state = api.read_state(commit)
            if state.get("active") is None and state.get("baseline"):
                raise OSError("Simulated response loss after accepted final state")
        api.update_ref = response_lost
        first = self.runner(store, manifest); transaction = first.acquire()
        with self.assertRaises(OSError):
            first.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        self.assertEqual(store.read()[1]["baseline"], transaction)
        api.update_ref = original
        retry = self.runner(store, manifest, 2)
        retry.acquire(verify_completed=lambda row: row["release"] == self.finish(manifest, row["proof"]))
        self.assertEqual(retry.execute(lambda row: self.fail("must not reread"), registry, self.proof, self.finish), transaction)
        self.assertEqual(registry.writes, ["sdk", "core"])

    def test_completed_json_alone_does_not_authorize_an_idempotent_success(self):
        api, store, manifest, bodies, registry = fixture()
        first = self.runner(store, manifest); first.acquire()
        first.execute(lambda row: bodies[row["name"]], registry, self.proof, self.finish)
        with self.assertRaisesRegex(ValueError, "completed publication confirmation"):
            self.runner(store, manifest, 2).acquire()

    def test_recovery_preserves_public_acceptance_and_finalization_phase(self):
        for phase in ['public-acceptance','release-finalization']:
            api,store,manifest,bodies,registry=fixture()
            first=self.runner(store,manifest);first.acquire();first.upload(lambda row:bodies[row['name']],registry)
            if phase=='release-finalization':
                def crash(value,proof):raise RuntimeError('response lost after tag')
                with self.assertRaises(RuntimeError):first.finalize(self.proof,crash)
            before=store.read()[1]['active']['packages'].copy()
            retry=self.runner(store,manifest,2);retry.recover(lambda owner:owner=={'runId':7,'attempt':1})
            active=store.read()[1]['active'];self.assertEqual(active['phase'],phase);self.assertEqual(active['packages'],before)
            retry.finalize(self.proof,self.finish)
            self.assertEqual(registry.writes,['sdk','core']);self.assertIsNotNone(store.read()[1]['baseline'])

    def test_recovery_cannot_move_to_a_new_run_or_steal_running_attempt(self):
        api,store,manifest,bodies,registry=fixture();self.runner(store,manifest).acquire()
        other=module.Publication(store,manifest,{'runId':8,'attempt':2},lambda value:True)
        with self.assertRaisesRegex(ValueError,'termination is unproven'):other.recover(lambda owner:True)
        with self.assertRaisesRegex(ValueError,'termination is unproven'):self.runner(store,manifest,2).recover(lambda owner:False)

    def test_ci_profile_or_chain_cannot_publish(self):
        for key in ["chain", "profile"]:
            api, store, manifest, bodies, registry = fixture(); manifest[key] = "ci"
            with self.assertRaisesRegex(ValueError, "Release chain"):
                self.runner(store, manifest)

    def test_structurally_valid_plan_does_not_authorize_itself(self):
        api, store, manifest, bodies, registry = fixture()
        with self.assertRaisesRegex(ValueError, "authorization missing"):
            self.runner(store, manifest, authorize=lambda value: False)

    def test_archive_changed_between_initial_check_and_upload_is_rejected(self):
        api, store, manifest, bodies, registry = fixture()
        runner = self.runner(store, manifest); runner.acquire(); reads = {}
        def read(row):
            name = row["name"]; reads[name] = reads.get(name, 0) + 1
            return bodies[name] if reads[name] == 1 else b"changed after validation"
        with self.assertRaisesRegex(ValueError, "changed before upload"):
            runner.execute(read, registry, self.proof, self.finish)
        self.assertEqual(registry.writes, [])

    def test_http_adapter_writes_single_state_blob_and_never_forces(self):
        calls = []
        def request(method, route, data, **options):
            calls.append((method, route, data))
            return {"sha": "a" * 40}
        api = module.GitHubStateAPI(request)
        commit = api.create_state_commit({"baseline": None, "active": None}, ["b" * 40])
        api.update_ref(api.REF, commit, False)
        self.assertEqual(calls[-1], ("PATCH", "/git/refs/heads/farm-release-state", {"sha": "a" * 40, "force": False}))
        self.assertEqual(calls[1][2]["tree"][0]["path"], "publication-state.json")
        self.assertEqual(calls[2][2]["parents"], ["b" * 40])
        with self.assertRaisesRegex(ValueError, "Forced"):
            api.update_ref(api.REF, commit, True)
        with self.assertRaisesRegex(ValueError, "Unexpected"):
            api.create_ref("heads/main", commit)

    def test_http_adapter_reads_exact_git_blob_bytes(self):
        body = json.dumps({"baseline": None, "active": None}).encode()
        identity = hashlib.sha1(b"blob " + str(len(body)).encode() + b"\0" + body).hexdigest()
        responses = [{"tree": {"sha": "b" * 40}}, {"truncated": False, "tree": [{"path": "publication-state.json", "mode": "100644", "type": "blob", "sha": identity}]},
                     {"encoding": "base64", "size": len(body), "content": base64.b64encode(body).decode()}]
        def request(method, route, data, **options):
            return responses.pop(0)
        self.assertEqual(module.GitHubStateAPI(request).read_state("a" * 40), {"baseline": None, "active": None})

    def test_http_adapter_rejects_wrong_blob_identity_and_other_ref(self):
        body = b'{}'
        responses = [{"tree": {"sha": "b" * 40}}, {"truncated": False, "tree": [{"path": "publication-state.json", "mode": "100644", "type": "blob", "sha": "c" * 40}]},
                     {"encoding": "base64", "size": 2, "content": base64.b64encode(body).decode()}]
        api = module.GitHubStateAPI(lambda *args, **options: responses.pop(0))
        with self.assertRaisesRegex(ValueError, "Git identity differs"):
            api.read_state("a" * 40)
        with self.assertRaisesRegex(ValueError, "Unexpected"):
            api.read_ref("heads/main")


if __name__ == "__main__":
    unittest.main(verbosity=2)
