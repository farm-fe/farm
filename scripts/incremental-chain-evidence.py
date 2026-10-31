"""Actions-only baseline lookup. Artifact JSON cannot supply its own trust anchor."""
# cspell:words importlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import stat
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


chain_plan = load("evidence_chain_plan", "incremental-chain-plan.py")
transport_module = load("evidence_transport", "ci-native-evidence.py")
registry_module = load("independent_registry", "incremental-registry-evidence.py")
WORKFLOWS = {"ci": ".github/workflows/ci.yaml", "release": ".github/workflows/incremental-release.yaml"}
PRODUCERS = {"ci": ("CI execution baseline", "Upload trusted CI execution baseline"),
             "release": ("Complete published release ledger", "Upload trusted published release ledger")}
MAX_LEDGER_BYTES = 4 * 1024 * 1024


def check(condition, message):
    if not condition:
        raise ValueError(message)


def policy_hash(files, modes):
    # Include all executable policy, not only the adapter's own file. Unknown
    # policy additions invalidate the baseline even if its JSON repeats old hashes.
    names = [name for name in files if name.startswith(("scripts/", ".github/", ".cargo/"))
             or name in chain_plan.impact.GLOBAL]
    check(names, "Execution policy inputs missing")
    records = chain_plan.snapshot_records(files, modes)
    return chain_plan.digest({name: records[name] for name in sorted(names)})


def source_reader(commit):
    chain_plan.impact.revision(commit)
    files, hashes = chain_plan.impact.snapshot(commit)
    tree = chain_plan.impact.git("rev-parse", commit + "^{tree}").decode("ascii").strip()
    modes = {name: row["mode"] for name, row in hashes.items()}
    return {"sha": commit, "tree": tree, "files": files, "modes": modes, "policyHash": policy_hash(files, modes),
            "parents": chain_plan.impact.git("show", "-s", "--format=%P", commit).decode("ascii").split()}


def ledger_member(body, artifact):
    check(0 < len(body) <= MAX_LEDGER_BYTES and len(body) == artifact["size_in_bytes"], "Baseline artifact size differs")
    check(artifact.get("digest") == "sha256:" + hashlib.sha256(body).hexdigest(), "Baseline artifact digest differs")
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries = archive.infolist()
        check(len(entries) == 1 and entries[0].filename == "ledger.json", "Baseline artifact members differ")
        entry = entries[0]
        check(stat.S_IFMT(entry.external_attr >> 16) in [0, stat.S_IFREG] and not entry.is_dir(), "Baseline artifact is not a regular file")
        check(0 < entry.file_size <= MAX_LEDGER_BYTES, "Expanded baseline exceeds bound")
        return json.loads(archive.read(entry).decode("utf-8"))


def successful_job(jobs, name):
    check(isinstance(name, str) and name, "Exact baseline job name missing")
    matches = [job for job in jobs if job["name"] == name or job["name"].endswith(" / " + name)]
    check(len(matches) == 1, "Missing or ambiguous baseline job")
    job = matches[0]
    check(job["status"] == "completed" and job["conclusion"] == "success", "Baseline job did not succeed")
    return job


def package_input_hashes(source):
    nodes, issues = chain_plan.repository_graph(source["files"])
    check(not issues, "Unresolved producer dependency graph")
    family = chain_plan.independent_family(source["files"], nodes)
    hashes = chain_plan.snapshot_records(source["files"], source["modes"])
    result = {}
    for identity, record in family.items():
        directory = str(Path(record["manifestPath"]).parent).replace("\\", "/")
        key = record["kind"] + ":" + directory
        pending = [key]
        if record.get("owner"):
            owner = family["npm:" + record["owner"]]
            pending.append("npm:" + str(Path(owner["manifestPath"]).parent).replace("\\", "/"))
        closure = set()
        while pending:
            name = pending.pop()
            if name not in closure:
                closure.add(name)
                pending.extend(nodes.get(name, {}).get("productDeps", []))
        prefixes = [name.split(":", 1)[1] + "/" for name in closure]
        selected = {file: row for file, row in hashes.items()
                    if any(file.startswith(prefix) for prefix in prefixes) or file in chain_plan.impact.GLOBAL
                    or file in chain_plan.impact.BUILD_POLICY or file.startswith((".cargo/", "patches/"))}
        check(selected, "Producer input closure empty")
        result[identity] = chain_plan.digest(selected)
    return result


def verify_ledger(ledger, chain, repository, run, artifact, jobs, source, trusted, expected_jobs, registry_checksums=None, measured_contexts=None):
    """All non-ledger arguments are independently fetched by BaselineAdapter."""
    check(chain in WORKFLOWS, "Unknown chain")
    check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"], "Baseline repository differs or is a fork")
    check(run["path"] == WORKFLOWS[chain], "Baseline workflow differs")
    check(run["event"] in ["push", "workflow_dispatch"] and run["head_branch"] == repository["default_branch"], "Only trusted default-branch baselines are reusable")
    check(run["status"] == "completed" and run["conclusion"] == "success", "Baseline run is incomplete or failed")
    check(run["head_sha"] == source["sha"], "Baseline checkout differs from its run")
    check(source["policyHash"] == trusted["policyHash"], "Baseline execution policy differs from trusted default branch")
    check(artifact["workflow_run"]["id"] == run["id"] and artifact["expired"] is False, "Baseline artifact expired or belongs to another run")
    expected_name = "farm-" + chain + "-baseline-" + source["sha"] + "-" + str(run["run_attempt"])
    check(artifact["name"] == expected_name, "Baseline artifact namespace differs")
    for key, value in {"chain": chain, "profile": chain_plan.CHAINS[chain], "policy": chain_plan.POLICY,
                       "repositoryId": repository["id"], "runId": run["id"], "runAttempt": run["run_attempt"],
                       "sourceSHA": source["sha"], "sourceTree": source["tree"], "policyHash": source["policyHash"]}.items():
        check(ledger.get(key) == value, "Baseline source or execution identity differs")
    check(expected_jobs and len(expected_jobs) == len(set(expected_jobs)), "Trusted acceptance inventory missing")
    check(sorted(ledger.get("requiredJobs", [])) == sorted(expected_jobs), "Baseline acceptance inventory differs")
    verified = [successful_job(jobs, name) for name in expected_jobs]
    producer_name, upload_name = PRODUCERS[chain]
    check(ledger.get("producerJob") == producer_name, "Ledger cannot select its own producer job")
    producer = successful_job(jobs, producer_name)
    check(all(job["run_id"] == run["id"] for job in [*verified, producer]), "Baseline jobs belong to another run")
    check(producer["started_at"] <= artifact["created_at"] <= producer["completed_at"], "Baseline artifact is outside its producer attempt")
    uploads = [step for step in producer.get("steps", []) if step.get("name") == upload_name]
    check(len(uploads) == 1 and uploads[0]["status"] == "completed" and uploads[0]["conclusion"] == "success",
          "Trusted producer upload step did not succeed")
    check(uploads[0]["started_at"] <= artifact["created_at"] <= uploads[0]["completed_at"], "Artifact is outside the trusted upload step")
    if chain == "release":
        check("Public registry consumers" in expected_jobs and ledger.get("registryVerified") is True, "Release has no completed public consumer proof")
        check(isinstance(registry_checksums, dict) and set(registry_checksums) == set(ledger.get("packages", {})), "Independent live registry byte evidence missing")
    inputs = package_input_hashes(source)
    check(set(ledger.get("packages", {})) == set(inputs), "Baseline producer package coverage differs")
    declared_family = json.loads(source["files"]["docs/releases/v2-candidate.json"])
    check(ledger.get("abi") == declared_family.get("abi"), "Baseline ABI differs from committed policy")
    locks = chain_plan.snapshot_records(source["files"], source["modes"])
    dependency_hash = chain_plan.digest({file: locks[file] for file in ["Cargo.lock", "pnpm-lock.yaml"] if file in locks})
    for identity, row in ledger["packages"].items():
        context = row.get("buildContext", {})
        check(isinstance(measured_contexts, dict) and measured_contexts.get(identity) == context,
              "Independent authenticated producer measurement missing or differs")
        check(context.get("chain") == chain and context.get("profile") == chain_plan.CHAINS[chain], "Producer context belongs to another chain")
        check(context.get("sourceInputHash") == inputs[identity] and context.get("recipeHash") == source["policyHash"], "Producer inputs or recipe differ")
        check(context.get("completeMeasuredContext") is True, "Producer environment was not completely measured")
        for key in ["toolchainHash", "dependencyHash", "flagsHash"]:
            chain_plan.exact(context.get(key), 64)
        check(context["dependencyHash"] == dependency_hash, "Producer dependency fingerprint differs")
        check(context.get("abi") == ledger.get("abi"), "Producer Farm ABI differs")
    anchor = {key: ledger[key] for key in ["chain", "repositoryId", "runId", "runAttempt", "sourceSHA", "sourceTree"]}
    anchor.update({"verified": True, "ledgerSha256": chain_plan.digest(ledger),
                   "registryChecksums": registry_checksums if chain == "release" else {}, "artifactId": artifact["id"],
                   "artifactDigest": artifact["digest"], "policySHA": trusted["sha"]})
    chain_plan.validate_baseline(ledger, anchor, chain, source["files"], source["modes"])
    return anchor


class BaselineAdapter:
    """The production transport forbids local use and never follows auth redirects."""
    def __init__(self, transport, repository_id, trusted_sha, chain, required_jobs, read_source=source_reader,
                 registry=None, measurement_reader=None):
        check(chain in WORKFLOWS, "Unknown baseline chain")
        chain_plan.exact(trusted_sha, 40)
        self.transport, self.repository_id, self.trusted_sha = transport, repository_id, trusted_sha
        self.chain, self.required_jobs, self.read_source = chain, required_jobs, read_source
        self.registry, self.measurement_reader = registry, measurement_reader

    def lookup(self):
        repository = self.transport.get("")
        check(repository["id"] == self.repository_id, "Repository identity differs")
        branch = self.transport.get("/branches/" + transport_module.urllib.parse.quote(repository["default_branch"], safe=""))
        check(branch["commit"]["sha"] == self.trusted_sha, "Trusted main moved before baseline lookup")
        trusted = self.read_source(self.trusted_sha)
        workflow = Path(WORKFLOWS[self.chain]).name
        runs = self.transport.get("/actions/workflows/" + workflow + "/runs?per_page=3&status=success&branch="
                                  + transport_module.urllib.parse.quote(repository["default_branch"], safe=""))["workflow_runs"]
        for candidate in runs[:3]:
            attempt = candidate["run_attempt"]
            run = self.transport.get("/actions/runs/" + str(candidate["id"]) + "/attempts/" + str(attempt))
            check(run["id"] == candidate["id"] and run["run_attempt"] == attempt, "Baseline attempt changed")
            source = self.read_source(run["head_sha"])
            wanted = "farm-" + self.chain + "-baseline-" + source["sha"] + "-" + str(attempt)
            # Exact name filtering avoids scanning hundreds of native artifacts.
            response = self.transport.get("/actions/runs/" + str(run["id"]) + "/artifacts?per_page=2&name="
                                          + transport_module.urllib.parse.quote(wanted, safe=""))
            matches = [row for row in response["artifacts"] if row["name"] == wanted and row["expired"] is False]
            if not matches:
                continue
            check(len(matches) == 1 and response.get("total_count", len(matches)) == 1, "Ambiguous baseline artifact")
            artifact = self.transport.get("/actions/artifacts/" + str(matches[0]["id"]))
            check(0 < artifact["size_in_bytes"] <= MAX_LEDGER_BYTES, "Baseline artifact exceeds bound")
            jobs = []
            for page in range(1, 4):
                batch = self.transport.get("/actions/runs/" + str(run["id"]) + "/attempts/" + str(attempt)
                                           + "/jobs?per_page=100&page=" + str(page))["jobs"]
                jobs.extend(batch)
                if len(batch) < 100:
                    break
            else:
                raise ValueError("Baseline job inventory exceeded bound")
            ledger = ledger_member(self.transport.archive(artifact["id"]), artifact)
            # A ledger flag or hex digest is not a measurement. No default reader
            # copies it back as proof; until the real producer reader is wired,
            # this baseline is rejected and the chain rebuilds.
            check(callable(self.measurement_reader), "Authenticated build measurement reader not available")
            measured = self.measurement_reader(run, jobs, source, trusted)
            checksums = None
            if self.chain == "release":
                registry = self.registry or registry_module.RegistryReadOnly()
                checksums = registry.verify(ledger["packages"])
            anchor = verify_ledger(ledger, self.chain, repository, run, artifact, jobs, source, trusted, self.required_jobs, checksums, measured)
            # Recheck the trusted policy after all network reads. The returned
            # binding is later compared again inside the publication lock.
            final = self.transport.get("/branches/" + transport_module.urllib.parse.quote(repository["default_branch"], safe=""))
            check(final["commit"]["sha"] == self.trusted_sha, "Trusted main moved during baseline lookup")
            return {"ledger": ledger, "anchor": anchor, "source": source,
                    "bindingHash": chain_plan.digest({"ledger": anchor["ledgerSha256"], "artifact": anchor["artifactDigest"],
                                                     "policySHA": self.trusted_sha, "run": run["id"], "attempt": attempt})}
        return None


def recheck_binding(first, current):
    check((first is None) == (current is None), "Successful baseline changed after planning")
    if first is not None:
        check(first["bindingHash"] == current["bindingHash"], "Successful baseline changed after planning")


def choose_plan(adapter, chain, source, files, modes, family, changes):
    """A failed lookup cannot authorize reuse, publication, or a smaller build."""
    diagnostic = None
    try:
        found = adapter.lookup()
    except Exception as error:
        # A malformed/expired/denied baseline must never shrink a build. No
        # exception text, response content or credentials enter the output.
        found = None
        diagnostic = "evidence-access-denied" if isinstance(error, PermissionError) else "evidence-invalid-or-unavailable"
    if found is None:
        plan = chain_plan.plan_chain(chain, source, files, family, changes, source_modes=modes)
        plan["baselineLookup"] = "unavailable-rebuild-within-current-chain"
        plan["baselineBinding"] = None
    else:
        plan = chain_plan.plan_chain(chain, source, files, family, changes, found["ledger"], found["source"]["files"], found["anchor"],
                                     source_modes=modes, baseline_modes=found["source"]["modes"])
        plan["baselineLookup"] = "verified-same-chain"
        plan["baselineBinding"] = found["bindingHash"]
    plan["lookupDiagnostic"] = diagnostic
    plan.pop("planHash", None)
    plan["planHash"] = chain_plan.digest(plan)
    return plan, found
