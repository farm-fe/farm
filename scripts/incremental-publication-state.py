"""Durable release state machine; all external writes require an owned lock.

The store must implement an atomic compare-and-swap of a durable revision.
No timeout steals a lock. A failed owner must release it, or an independent
terminal-run check must authorize recovery of the same frozen transaction.
"""
import copy
import base64
import hashlib
import json
import re
import uuid


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def frozen_manifest(manifest):
    require(manifest.get("chain") == "release" and manifest.get("profile") == "release", "Release chain and profile required")
    for key, length in [("sourceSHA", 40), ("sourceTree", 40), ("planHash", 64)]:
        require(bool(re.fullmatch("[a-f0-9]{" + str(length) + "}", manifest.get(key, ""))), "Frozen source and plan required")
    require("baseline" in manifest, "Explicit baseline required")
    rows = manifest.get("packages")
    require(isinstance(rows, list) and bool(rows), "Nonempty exact publication inventory required")
    names = set()
    for row in rows:
        require(row.get("kind") in ["cargo", "npm"], "Unknown package kind")
        key = (row["kind"], row.get("name"))
        require(isinstance(key[1], str) and bool(key[1]) and key not in names, "Duplicate publication package")
        names.add(key)
        require(bool(re.fullmatch(r"\d+\.\d+\.\d+", row.get("version", ""))), "Exact stable version required")
        require(bool(re.fullmatch("[a-f0-9]{64}", row.get("archiveSha256", ""))), "Exact original archive required")
        require(type(row.get("archiveBytes")) is int and row["archiveBytes"] > 0, "Archive byte count required")
    return digest(manifest)


class Publication:
    """Store read returns (revision, state); swap must reject a stale revision.

    Registry callbacks inspect real bytes independently. Saved per-package
    status is diagnostic: every resume rechecks registry and original archives.
    """
    def __init__(self, store, manifest, owner, verify_manifest):
        self.store, self.manifest, self.owner = store, copy.deepcopy(manifest), copy.deepcopy(owner)
        self.transaction = frozen_manifest(self.manifest)
        self.fence, self.acquired, self.completed = uuid.uuid4().hex, False, False
        require(type(owner.get("runId")) is int and owner["runId"] > 0
                and type(owner.get("attempt")) is int and owner["attempt"] > 0, "Actual run identity required")
        # A structural hash cannot authorize publishing. This callback must
        # independently verify source, plan, release producer and full inventory.
        require(verify_manifest(self.manifest) is True, "Independent manifest authorization missing")

    def acquire(self, terminal_owner=lambda owner: False, verify_completed=lambda receipt: False):
        revision, state = self.store.read()
        state = copy.deepcopy(state)
        if state.get("baseline") == self.transaction and state.get("active") is None:
            completed = state.get("completed", {}).get(self.transaction)
            require(isinstance(completed, dict) and completed.get("manifest") == self.manifest
                    and completed.get("proof", {}).get("manifestHash") == self.transaction
                    and completed.get("release", {}).get("manifestHash") == self.transaction
                    and completed.get("release", {}).get("sourceSHA") == self.manifest["sourceSHA"]
                    and verify_completed(copy.deepcopy(completed)) is True,
                    "Independent completed publication confirmation missing")
            self.completed = True
            return self.transaction
        require(state.get("baseline") == self.manifest["baseline"], "Publication baseline moved; replan required")
        previous = state.get("active")
        if previous:
            require(previous["transaction"] == self.transaction, "Unfinished publication requires its original manifest")
            require(previous["owner"] != self.owner and terminal_owner(previous["owner"]) is True,
                    "Publication owner is still active or its termination is unproven")
        state["active"] = {"transaction": self.transaction, "manifest": copy.deepcopy(self.manifest),
                           "owner": self.owner, "authorization": copy.deepcopy(previous.get("authorization") if previous else self.owner),
                           "fence": self.fence, "handoff": None,
                           "packages": previous["packages"] if previous else {}, "phase": "uploading"}
        self.revision = self.store.swap(revision, state)
        self.acquired = True
        return self.transaction

    def owned(self):
        revision, state = self.store.read()
        active = state.get("active")
        require(self.acquired and active is not None and active["owner"] == self.owner
                and active["transaction"] == self.transaction and active.get("fence") == self.fence,
                "Publication lock ownership lost")
        require(active["manifest"] == self.manifest and state["baseline"] == self.manifest["baseline"],
                "Frozen publication inputs changed")
        return revision, copy.deepcopy(state)

    def handoff(self, target):
        require(target in ["public-acceptance", "release-finalization"], "Unknown trusted handoff stage")
        revision, state = self.owned()
        require(state["active"]["phase"] == target, "Publication handoff phase differs")
        state["active"]["fence"] = None
        state["active"]["handoff"] = target
        self.store.swap(revision, state)
        self.acquired = False

    def claim_handoff(self, target):
        # The workflow adapter must independently verify this job has the
        # trusted stage role; run identity alone does not establish that role.
        require(target in ["public-acceptance", "release-finalization"], "Unknown trusted handoff stage")
        revision, state = self.store.read(); state = copy.deepcopy(state)
        active = state.get("active")
        require(active is not None and active["owner"] == self.owner and active["transaction"] == self.transaction
                and active["manifest"] == self.manifest and active.get("fence") is None
                and active.get("handoff") == target and active["phase"] == target,
                "Exact pending publication handoff missing")
        require(state["baseline"] == self.manifest["baseline"], "Publication baseline moved")
        active["fence"], active["handoff"] = self.fence, None
        self.store.swap(revision, state)
        self.acquired = True

    def recover(self, terminal_owner, verify_completed=lambda receipt: False):
        """Preserve the frozen producer, package states and publication phase.

        Only a later attempt of the same workflow run may replace a terminated
        writer. No lease timeout, changed source or new run grants ownership.
        """
        revision, state = self.store.read(); state = copy.deepcopy(state)
        active = state.get('active')
        if active is None:
            return self.acquire(terminal_owner, verify_completed)
        require(active.get('transaction') == self.transaction and active.get('manifest') == self.manifest,
                'Unfinished publication requires its original manifest')
        old = active.get('owner', {})
        require(old.get('runId') == self.owner['runId'] and old.get('attempt', 0) < self.owner['attempt']
                and terminal_owner(copy.deepcopy(old)) is True, 'Original publication attempt termination is unproven')
        require(state.get('baseline') == self.manifest['baseline'], 'Publication baseline moved; replan required')
        require(active.get('phase') in ['uploading', 'public-acceptance', 'release-finalization'], 'Unknown recovery phase')
        active.update({'owner': self.owner, 'fence': self.fence, 'handoff': None})
        self.store.swap(revision, state); self.acquired = True
        return self.transaction

    def reject_unwritten(self, reason="locked-plan-mismatch-before-registry-write"):
        """A mismatched plan may end only before any registry upload was started."""
        require(reason in ['locked-plan-mismatch-before-registry-write', 'locked-plan-unavailable-before-registry-write'], 'Unknown unwritten rejection reason')
        revision, state = self.owned(); active = state['active']
        require(active['phase'] == 'uploading' and not active['packages'], 'A started publication must retain its frozen transaction')
        state.setdefault('rejected', {})[self.transaction] = {'manifestHash': self.transaction, 'sourceSHA': self.manifest['sourceSHA'],
                                                            'reason': reason}
        state['active'] = None
        self.store.swap(revision, state); self.acquired = False

    def record(self, key, status):
        revision, state = self.owned()
        state["active"]["packages"][key] = status
        self.revision = self.store.swap(revision, state)

    def upload(self, archive_reader, registry):
        self.owned()
        # Never rebuild on resume. Verify every original byte before any upload.
        for row in self.manifest["packages"]:
            body = archive_reader(row)
            require(len(body) == row["archiveBytes"] and hashlib.sha256(body).hexdigest() == row["archiveSha256"],
                    "Original sealed archive differs; resume refused")
        # Detect a conflict anywhere before writing even the first dependency.
        # This preflight is not a reservation; recheck each version at upload.
        for row in self.manifest["packages"]:
            self.owned()
            actual = registry.lookup(row)
            require(actual is None or actual == row["archiveSha256"], "Immutable registry version has different bytes")
        # Cargo precedes npm; the independently authorized manifest preserves
        # dependency order within each kind.
        rows = sorted(self.manifest["packages"], key=lambda row: row["kind"] != "cargo")
        for row in rows:
            self.owned()
            key = row["kind"] + ":" + row["name"]
            actual = registry.lookup(row)
            if actual is not None:
                require(actual == row["archiveSha256"], "Immutable registry version has different bytes")
            else:
                self.record(key, "upload-started")
                self.owned()
                body = archive_reader(row)
                require(len(body) == row["archiveBytes"] and hashlib.sha256(body).hexdigest() == row["archiveSha256"],
                        "Original sealed archive changed before upload")
                registry.publish(row, body)
                require(registry.lookup(row) == row["archiveSha256"], "Registry byte confirmation missing")
            if hasattr(registry, "confirm"):
                self.owned()
                registry.confirm(row)
            self.record(key, "registry-confirmed")
        revision, state = self.owned()
        state["active"]["phase"] = "public-acceptance"
        self.store.swap(revision, state)

    def finalize(self, public_acceptance, release_finalize):
        self.owned()
        _, state = self.owned()
        require(state["active"]["phase"] in ["public-acceptance", "release-finalization"], "Uploads not confirmed")
        expected = {row["kind"] + ":" + row["name"] for row in self.manifest["packages"]}
        require(set(state["active"]["packages"]) == expected
                and all(value == "registry-confirmed" for value in state["active"]["packages"].values()),
                "Incomplete package publication inventory")
        # Lock remains owned through installation, ABI, checksum and tags checks.
        proof = public_acceptance(copy.deepcopy(self.manifest))
        require(isinstance(proof, dict) and proof.get("manifestHash") == self.transaction
                and proof.get("verified") is True, "Independent public acceptance missing")
        revision, state = self.owned()
        state["active"]["phase"] = "release-finalization"
        state["active"]["publicProof"] = proof
        self.store.swap(revision, state)
        # This operation must read back an existing tag/Release on retry. A
        # crash after its write cannot authorize changing its target or body.
        final = release_finalize(copy.deepcopy(self.manifest), copy.deepcopy(proof))
        require(isinstance(final, dict) and final.get("manifestHash") == self.transaction
                and final.get("sourceSHA") == self.manifest["sourceSHA"] and final.get("verified") is True,
                "Exact tag and Release confirmation missing")
        revision, state = self.owned()
        state.setdefault("completed", {})[self.transaction] = {"manifest": self.manifest, "proof": proof, "release": final,
                                                                  "authorization": copy.deepcopy(state["active"].get("authorization"))}
        state["baseline"] = self.transaction
        state["active"] = None
        self.store.swap(revision, state)
        return self.transaction

    def execute(self, archive_reader, registry, public_acceptance, release_finalize):
        if self.completed:
            return self.transaction
        self.upload(archive_reader, registry)
        return self.finalize(public_acceptance, release_finalize)


class GitRefStore:
    """Atomic durable store using an existing authorized GitHub Git-data API.

    Every proposed commit has the previously read commit as its parent. A
    non-forced ref update rejects concurrent sibling commits. Ref creation also
    rejects races. The adapter must use one fixed repository/ref and must never
    retry a rejected update against a newly read parent automatically.
    """
    def __init__(self, api, reference="heads/farm-release-state"):
        self.api, self.reference = api, reference

    def read(self):
        commit = self.api.read_ref(self.reference)
        if commit is None:
            return None, {"baseline": None, "active": None, "completed": {}}
        state = self.api.read_state(commit)
        require(isinstance(state, dict) and "baseline" in state and "active" in state, "Invalid durable publication state")
        return commit, state

    def swap(self, expected, state):
        # Git ref updates are ancestry checks, not general CAS. A fresh nonce
        # makes concurrent same-state proposals distinct sibling commits, even
        # with identical authors/timestamps. Never rebase a rejected proposal.
        proposed = copy.deepcopy(state)
        proposed["operationId"] = uuid.uuid4().hex
        commit = self.api.create_state_commit(proposed, parents=[] if expected is None else [expected])
        if expected is None:
            self.api.create_ref(self.reference, commit)
        else:
            self.api.update_ref(self.reference, commit, force=False)
        return commit


class GitHubStateAPI:
    """Git-data transport adapter; request is an existing authenticated client.

    The client must reject HTTP failures and only accept the fixed repository.
    Missing refs must be represented by None, never by swallowing other errors.
    No token is written to Git state. This adapter never modifies main.
    """
    REF = "heads/farm-release-state"
    FILE = "publication-state.json"
    LIMIT = 4 * 1024 * 1024

    def __init__(self, request):
        self.request = request

    def read_ref(self, reference):
        require(reference == self.REF, "Unexpected publication state ref")
        response = self.request("GET", "/git/ref/" + reference, None, missing=True)
        if response is None:
            return None
        require(response.get("object", {}).get("type") == "commit", "Publication ref must point to commit")
        return self.sha(response["object"]["sha"])

    @staticmethod
    def sha(value):
        require(isinstance(value, str) and bool(re.fullmatch("[a-f0-9]{40}", value)), "Invalid Git object identity")
        return value

    def read_state(self, commit):
        source = self.request("GET", "/git/commits/" + self.sha(commit), None)
        tree = self.request("GET", "/git/trees/" + self.sha(source["tree"]["sha"]), None)
        require(tree.get("truncated") is False and len(tree.get("tree", [])) == 1, "Unexpected publication tree")
        entry = tree["tree"][0]
        require(entry.get("path") == self.FILE and entry.get("mode") == "100644" and entry.get("type") == "blob",
                "Publication state must be one regular JSON blob")
        blob = self.request("GET", "/git/blobs/" + self.sha(entry["sha"]), None)
        require(blob.get("encoding") == "base64" and type(blob.get("size")) is int
                and 0 < blob["size"] <= self.LIMIT, "Publication state blob exceeds bound")
        encoded = "".join(blob["content"].splitlines())
        require(len(encoded) <= 4 * ((self.LIMIT + 2) // 3), "Encoded publication state exceeds bound")
        body = base64.b64decode(encoded, validate=True)
        require(len(body) == blob["size"], "Publication blob size differs")
        actual = hashlib.sha1(b"blob " + str(len(body)).encode() + b"\0" + body).hexdigest()
        require(actual == entry["sha"], "Publication blob Git identity differs")
        return json.loads(body.decode("utf-8"))

    def create_state_commit(self, state, parents):
        require(len(parents) <= 1, "Publication state has at most one parent")
        for parent in parents:
            self.sha(parent)
        body = json.dumps(state, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()
        require(0 < len(body) <= self.LIMIT, "Publication state exceeds bound")
        blob = self.request("POST", "/git/blobs", {"encoding": "base64", "content": base64.b64encode(body).decode("ascii")})
        tree = self.request("POST", "/git/trees", {"tree": [{"path": self.FILE, "mode": "100644", "type": "blob", "sha": self.sha(blob["sha"])}]})
        commit = self.request("POST", "/git/commits", {"message": "Record exact publication progress", "tree": self.sha(tree["sha"]), "parents": parents})
        return self.sha(commit["sha"])

    def create_ref(self, reference, commit):
        require(reference == self.REF, "Unexpected publication state ref")
        return self.request("POST", "/git/refs", {"ref": "refs/" + reference, "sha": self.sha(commit)})

    def update_ref(self, reference, commit, force):
        require(reference == self.REF and force is False, "Forced publication update forbidden")
        return self.request("PATCH", "/git/refs/" + reference, {"sha": self.sha(commit), "force": False})
