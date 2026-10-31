"""Shared impact rules; CI and release keep separate build and baseline identities."""
# cspell:words argparse importlib tomllib
import hashlib
import importlib.util
import json
import argparse
from pathlib import Path
import posixpath
import re
import tomllib

spec = importlib.util.spec_from_file_location("chain_impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec)
spec.loader.exec_module(impact)

POLICY = "farm-independent-chains-v1"
CHAINS = {"ci": "ci", "release": "release"}
SDK_ROOTS = ["farmfe_core", "farmfe_compiler", "farmfe_toolkit", "farmfe_ecosystem_glob"]


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()


def exact(value, size):
    if not isinstance(value, str) or not re.fullmatch("[a-f0-9]{" + str(size) + "}", value):
        raise ValueError("Exact source or content identity required")
    return value


def snapshot_records(files, modes):
    if not isinstance(modes, dict) or set(modes) != set(files) or any(mode not in ["100644", "100755"] for mode in modes.values()):
        raise ValueError("Complete Git file mode evidence required")
    return {name: {"mode": modes[name], "sha256": hashlib.sha256(body).hexdigest()} for name, body in sorted(files.items())}


def repository_graph(files):
    """Directory identities preserve duplicate private example names without merging them."""
    nodes, names, issues = {}, {}, []
    workspace = tomllib.loads(files["Cargo.toml"].decode("utf-8"))
    inherited = workspace.get("workspace", {}).get("dependencies", {})
    patterns = workspace["workspace"]["members"]
    lines = files["pnpm-workspace.yaml"].decode("utf-8").splitlines()
    if any(line.strip() and line.strip() != "packages:" and not line.lstrip().startswith("#")
           and not re.fullmatch(r"\s*-\s+[\w*/.-]+\s*", line) for line in lines):
        raise ValueError("Workspace configuration requires full execution")
    npm_patterns = [line.strip()[2:].strip().rstrip("/") for line in lines if line.strip().startswith("- ")]
    for file, body in sorted(files.items()):
        directory = posixpath.dirname(file)
        cargo = file.endswith("/Cargo.toml") and (directory.startswith("patches/") or any(impact.matches(directory, row) for row in patterns))
        npm = file.endswith("/package.json") and any(impact.matches(directory, row) for row in npm_patterns)
        if not cargo and not npm:
            continue
        manifest = tomllib.loads(body.decode("utf-8")) if cargo else json.loads(body)
        package = manifest.get("package", {}) if cargo else manifest
        if not isinstance(package.get("name"), str):
            raise ValueError("Workspace package identity missing")
        kind = "cargo" if cargo else "npm"
        key = kind + ":" + directory
        nodes[key] = {"kind": kind, "directory": directory, "manifest": file, "name": package["name"],
                      "version": package.get("version"), "data": manifest, "productDeps": set(), "testDeps": set()}
        names.setdefault(kind + ":" + package["name"], []).append(key)
    for key, node in nodes.items():
        manifest = node["data"]
        tables = [manifest, *manifest.get("target", {}).values()] if node["kind"] == "cargo" else [manifest]
        for table in tables:
            kinds = ["dependencies", "build-dependencies", "dev-dependencies"] if node["kind"] == "cargo" else ["dependencies", "optionalDependencies", "peerDependencies", "devDependencies"]
            for kind in kinds:
                for alias, declaration in table.get(kind, {}).items():
                    dep = declaration if isinstance(declaration, dict) else {}
                    origin = node["directory"]
                    if dep.get("workspace"):
                        source = inherited.get(alias, {})
                        dep = {**(source if isinstance(source, dict) else {}), **dep}
                        origin = ""
                    if node["kind"] == "cargo" and "path" in dep:
                        target = "cargo:" + posixpath.normpath(posixpath.join(origin, dep["path"]))
                        candidates = [target] if target in nodes else []
                        if not candidates:
                            issues.append("missing-path-dependency")
                    elif node["kind"] == "npm" and isinstance(declaration, str) and declaration.startswith(("file:", "link:")):
                        target = "npm:" + posixpath.normpath(posixpath.join(origin, declaration.split(":", 1)[1]))
                        candidates = [target] if target in nodes else []
                        if not candidates:
                            issues.append("missing-local-dependency")
                    else:
                        dependency_name = dep.get("package", alias)
                        if node["kind"] == "npm" and isinstance(declaration, str) and declaration.startswith("npm:"):
                            dependency_name = declaration[4:].rsplit("@", 1)[0]
                            issues.append("alias-dependency-requires-review")
                        if node["kind"] == "npm" and isinstance(declaration, str) and declaration.startswith("workspace:"):
                            local = declaration[len("workspace:"):]
                            if local.startswith((".", "/")):
                                target = "npm:" + posixpath.normpath(posixpath.join(origin, local))
                                candidates = [target] if target in nodes else []
                            elif not re.fullmatch(r"[v0-9*^~<>=|. +\-]+", local):
                                candidates = []
                                issues.append("workspace-alias-requires-review")
                            else:
                                candidates = names.get("npm:" + alias, [])
                            if not candidates:
                                issues.append("missing-local-dependency")
                        else:
                            candidates = names.get(node["kind"] + ":" + dependency_name, [])
                    if len(candidates) > 1:
                        issues.append("ambiguous-dependency")
                    for target in candidates:
                        node["testDeps"].add(target)
                        # JS build configs may load dev dependencies. Cargo test-only
                        # edges stay distinct from its production dependency graph.
                        if kind != "dev-dependencies":
                            node["productDeps"].add(target)
        if node["kind"] == "npm":
            directory = "crates/node" if node["directory"] == "packages/core" else node["directory"]
            bridge = "cargo:" + directory
            if bridge in nodes:
                node["productDeps"].add(bridge)
                node["testDeps"].add(bridge)
    # The shared Examples runner builds these prerequisites even when an
    # individual package.json omits them. They are acceptance edges, not a
    # reason to publish every Example or bump unrelated public packages.
    shared = {"npm:" + directory for directory in ["packages/core", "packages/cli", "packages/runtime",
              "packages/runtime-plugin-hmr", "packages/runtime-plugin-import-meta"] if "npm:" + directory in nodes}
    required_shared = {"npm:" + directory for directory in ["packages/core", "packages/cli", "packages/runtime",
                       "packages/runtime-plugin-hmr", "packages/runtime-plugin-import-meta"]}
    for node in nodes.values():
        if node["kind"] == "npm" and (node["directory"].startswith("examples/") or node["directory"] == "website"):
            if node['data'].get('scripts',{}).get('build') and shared != required_shared:
                issues.append('required-shared-test-root-missing')
            node["testDeps"].update(shared)
    return nodes, sorted(set(issues))


def independent_family(files, nodes):
    """Derive the full release family from manifests and the reviewed native recipe."""
    result = {}
    def add(kind, package, file, owner=None, platform=None):
        version = package.get("version")
        if version == {"workspace": True}:
            version = tomllib.loads(files["Cargo.toml"].decode("utf-8"))["workspace"]["package"]["version"]
        row = {"kind": kind, "name": package["name"], "version": version, "manifestPath": file}
        if owner is not None:
            row.update({"owner": owner, "platform": platform})
        key = kind + ":" + package["name"]
        if key in result:
            raise ValueError("Duplicate independently derived publication identity")
        result[key] = row
    recipe = files.get("scripts/ci-build-rust-artifacts.sh", b"").decode("utf-8")
    regular = re.search(r'^PLUGINS="([^"\n]+)"', recipe, re.M)
    docs = re.search(r'^\s+((?:win32|linux|darwin)[^\n)]+)\)\r?\n\s+PLUGINS="\$PLUGINS ([^"\n]+)"', recipe, re.M)
    for node in nodes.values():
        directory = node["directory"]
        if node["kind"] != "npm" or not re.fullmatch(r"(?:packages|rust-plugins|js-plugins)/[^/]+", directory):
            continue
        package = node["data"]
        # This existing independent package is intentionally outside the V2 family.
        if package.get("private") or package["name"] == "@farmfe/js-plugin-copy":
            continue
        add("npm", package, node["manifest"])
        if not package.get("napi"):
            continue
        platforms = impact.TARGETS
        if directory.startswith("rust-plugins/"):
            if not regular or not docs:
                raise ValueError("Reviewed native support recipe missing")
            plugin = directory.split("/")[1]
            if plugin in docs[2].split():
                platforms = docs[1].split("|")
            elif plugin not in regular[1].split():
                raise ValueError("Native owner is absent from the support recipe")
        for platform in platforms:
            file = directory + "/npm/" + platform + "/package.json"
            payload = json.loads(files[file])
            if payload["name"] != package["name"] + "-" + platform or payload["version"] != package["version"]:
                raise ValueError("Native support payload identity differs")
            if package.get("optionalDependencies", {}).get(payload["name"]) not in [package["version"], "workspace:*"]:
                raise ValueError("Native optional dependency is absent or differs")
            add("npm", payload, file, package["name"], platform)
    cargo = {node["name"]: key for key, node in nodes.items() if node["kind"] == "cargo"}
    if any(name not in cargo for name in SDK_ROOTS):
        raise ValueError("Complete SDK root set missing")
    pending, seen = [cargo[name] for name in SDK_ROOTS], set()
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        node = nodes[key]
        if node["directory"].startswith("patches/"):
            # Vendored upstream inputs affect builds but are never Farm SDK releases.
            continue
        if node["data"]["package"].get("publish") is False:
            raise ValueError("SDK dependency is explicitly non-publishable")
        add("cargo", node["data"]["package"], node["manifest"])
        pending.extend(node["productDeps"])
    return result


def union_graph(*graphs):
    result = {}
    for graph in graphs:
        for key, node in graph.items():
            old = result.get(key, {})
            result[key] = {**node, "productDeps": old.get("productDeps", set()) | node["productDeps"],
                           "testDeps": old.get("testDeps", set()) | node["testDeps"]}
    return result


def family_records(family):
    result = {}
    groups = [("cargo", family["cargo"]), ("npm", family["npm"]["primary"]), ("npm", family["npm"]["payloads"])]
    for kind, records in groups:
        for row in records:
            key = kind + ":" + row["name"]
            if key in result:
                raise ValueError("Duplicate publication identity")
            if not re.fullmatch(r"\d+\.\d+\.\d+", row.get("version", "")):
                raise ValueError("Stable explicit package version required")
            result[key] = {**row, "kind": kind}
    return result


def validate_family_sources(records, files):
    for row in records.values():
        file = row["manifestPath"]
        if not isinstance(file, str) or file.startswith("/") or any(part in ["", ".", ".."] for part in file.split("/")):
            raise ValueError("Unsafe package manifest path")
        manifest = tomllib.loads(files[file].decode("utf-8")) if row["kind"] == "cargo" else json.loads(files[file])
        package = manifest["package"] if row["kind"] == "cargo" else manifest
        version = package.get("version")
        if version == {"workspace": True}:
            version = tomllib.loads(files["Cargo.toml"].decode("utf-8"))["workspace"]["package"]["version"]
        if package.get("name") != row["name"] or version != row["version"]:
            raise ValueError("Publication inventory differs from actual source manifest")


def validate_baseline(ledger, anchor, chain, source_files, source_modes):
    """The caller must obtain anchor from trusted GitHub/registry evidence, never PR JSON."""
    if ledger is None:
        return None
    if not isinstance(anchor, dict) or anchor.get("verified") is not True:
        raise ValueError("Baseline has no independently verified anchor")
    if anchor.get("ledgerSha256") != digest(ledger) or anchor.get("chain") != chain:
        raise ValueError("Baseline anchor or chain differs")
    for key in ["repositoryId", "runId", "runAttempt"]:
        if not isinstance(ledger.get(key), int) or isinstance(ledger[key], bool) or ledger[key] <= 0 or ledger[key] != anchor.get(key):
            raise ValueError("Baseline execution identity differs")
    if ledger.get("schemaVersion") != 1 or ledger.get("chain") != chain or ledger.get("status") != "success":
        raise ValueError("Only a complete successful same-chain baseline is accepted")
    if ledger.get("profile") != CHAINS[chain] or ledger.get("policy") != POLICY:
        raise ValueError("Baseline profile or policy differs")
    if chain == "release" and ledger.get("registryVerified") is not True:
        raise ValueError("Release baseline is not a successfully published version")
    exact(ledger.get("sourceSHA"), 40); exact(ledger.get("sourceTree"), 40)
    if ledger["sourceSHA"] != anchor.get("sourceSHA") or ledger["sourceTree"] != anchor.get("sourceTree"):
        raise ValueError("Baseline source differs from trusted source")
    if ledger.get("sourceSnapshotHash") != digest(snapshot_records(source_files, source_modes)):
        raise ValueError("Baseline source snapshot differs")
    records = ledger.get("packages")
    if not isinstance(records, dict) or not records:
        raise ValueError("Baseline package coverage missing")
    old_nodes, _ = repository_graph(source_files)
    expected = independent_family(source_files, old_nodes)
    if set(records) != set(expected):
        raise ValueError("Baseline full family coverage differs")
    for key, row in records.items():
        if row.get("identity") != key:
            raise ValueError("Baseline package identity differs")
        exact(row.get("archiveSha256"), 64)
        if not re.fullmatch(r"\d+\.\d+\.\d+", row.get("version", "")):
            raise ValueError("Baseline package version missing")
        if row["version"] != expected[key]["version"]:
            raise ValueError("Baseline version differs from source")
        if chain == "release" and anchor.get("registryChecksums", {}).get(key) != row["archiveSha256"]:
            raise ValueError("Baseline package lacks independent registry checksum")
    return ledger


def plan_chain(chain, source, files, family, changes, baseline=None, baseline_files=None, anchor=None, *, source_modes, baseline_modes=None):
    """Executable selection calculation. It grants no upload or artifact trust by itself."""
    if chain not in CHAINS:
        raise ValueError("Unknown build chain")
    exact(source["sha"], 40); exact(source["tree"], 40)
    records = family_records(family)
    validate_family_sources(records, files)
    source_records = snapshot_records(files, source_modes)
    head, issues = repository_graph(files)
    independent = independent_family(files, head)
    if set(records) != set(independent) or any(any(row.get(key) != independent[name].get(key) for key in ["kind", "name", "version", "manifestPath", "owner", "platform"]) for name, row in records.items()):
        raise ValueError("Publication inventory does not cover the independently derived full family")
    metadata_file = "docs/releases/v2-candidate.json"
    if metadata_file in files:
        declared = json.loads(files[metadata_file])
        normalized = family_records(declared)
        fields = ["kind", "name", "version", "manifestPath", "owner", "platform"]
        if (declared.get("abi") != family.get("abi") or set(normalized) != set(records)
                or any(any(row.get(field) != normalized[name].get(field) for field in fields) for name, row in records.items())):
            raise ValueError("Committed family metadata differs from independently validated manifests")
    reasons, full, product_full = [], False, False
    validation_required = set()
    if baseline is None:
        full = True; reasons.append("no-successful-same-chain-baseline")
        old = {}
    else:
        validate_baseline(baseline, anchor, chain, baseline_files or {}, baseline_modes)
        old, old_issues = repository_graph(baseline_files)
        issues += old_issues
    graph = union_graph(old, head)
    roots = set()
    # Do not let an incomplete caller diff hide a changed baseline input. Renames
    # and deletions are represented by both old and new paths in this union.
    baseline_records = snapshot_records(baseline_files, baseline_modes) if baseline is not None else {}
    observed = sorted(name for name in set(baseline_records) | set(source_records)
                      if baseline is not None and baseline_records.get(name) != source_records.get(name))
    complete_changes = list(changes) + [{"status": "M", "paths": observed}]
    for change in complete_changes:
        for file in change["paths"]:
            if not isinstance(file, str) or file.startswith("/") or any(part in ["", ".", ".."] for part in file.split("/")):
                raise ValueError("Unsafe changed source path")
            if file == metadata_file:
                if file not in files:
                    raise ValueError("Committed family metadata was deleted")
                # This is a validated derived inventory, not product source.
                # Its contracts still run, but it cannot invent SDK code changes.
                reasons.append("validated-family-metadata-changed")
                validation_required.add("family-metadata-contracts")
                continue
            if file in impact.GLOBAL or file.startswith((".github/", ".cargo/", "patches/", "scripts/")):
                full = True; reasons.append("global-or-execution-policy-changed")
            if file in impact.GLOBAL or file.startswith((".cargo/", "patches/")):
                product_full = True
            names = set().union(impact.owners(old, file), impact.owners(head, file))
            roots.update(names)
            if not names and file not in impact.GLOBAL:
                full = True; reasons.append("unmapped-source-changed")
                if not file.startswith((".github/", "scripts/")):
                    product_full = True
    if issues:
        full = True; product_full = True; reasons += issues
    affected = set(graph) if product_full else set(impact.reverse_closure(graph, roots, "productDeps"))
    tests = set(graph) if full else set(impact.reverse_closure(graph, roots, "testDeps"))
    build, publish, bump, unchanged, collisions = [], [], [], [], []
    prior = baseline.get("packages", {}) if baseline else {}
    removed = sorted(set(prior) - set(records))
    abi_changed = baseline is not None and baseline.get("abi") != family.get("abi")
    owners = {row["owner"] for row in family["npm"]["payloads"] if row["owner"] != "create-farm"} if abi_changed else set()
    abi_roots = {key for key, node in graph.items() if node["kind"] == "npm" and node["name"] in owners}
    if abi_roots:
        affected.update(impact.reverse_closure(graph, abi_roots, "productDeps"))
        tests.update(impact.reverse_closure(graph, abi_roots, "testDeps"))
    if "npm:js-plugins/copy" in roots:
        raise ValueError("Changed independently excluded package requires explicit publication scope review")
    for identity, row in sorted(records.items()):
        key = row["kind"] + ":" + posixpath.dirname(row["manifestPath"])
        affected_package = key in affected or row.get("owner") in owners or row["name"] in owners
        if row.get("owner"):
            owner = records.get("npm:" + row["owner"])
            if not owner or owner["version"] != row["version"]:
                raise ValueError("Native owner and payload versions differ")
            affected_package |= "npm:" + posixpath.dirname(owner["manifestPath"]) in affected
        previous = prior.get(identity)
        if full or affected_package or previous is None:
            build.append(identity)
        if previous is None or previous.get("version") != row["version"]:
            publish.append(identity)
        elif affected_package:
            bump.append(identity)
            collisions.append(identity)
        else:
            unchanged.append(identity)
    result = {"schemaVersion": 1, "policy": POLICY, "chain": chain, "profile": CHAINS[chain],
              "namespace": "farm-" + chain + "-v1", "sourceSHA": source["sha"], "sourceTree": source["tree"],
              "baselineSHA": baseline.get("sourceSHA") if baseline else None,
              "sourceSnapshotHash": digest(source_records),
              "fullBuild": full, "reasons": sorted(set(reasons)), "abiChanged": abi_changed,
              "build": build, "versionRequired": bump if chain == "release" else [],
              "fullFamily": [records[key] for key in sorted(records)],
              "publish": publish if chain == "release" else [], "unchanged": unchanged,
              "testNodes": sorted(tests), "affectedNodes": sorted(affected),
              "removedPackages": removed,
              "publicationBlocked": bool(collisions or removed) if chain == "release" else True,
              "publicationBlockReason": "retirement-authorization-required" if removed else "changed-immutable-version" if collisions else None,
              "registryCollisionChecks": publish if chain == "release" else [],
              "validationRequired": sorted(validation_required),
              "crossChainReuseAllowed": False, "executionAuthorized": False, "publicationAuthorized": False}
    dependencies = set(build)
    by_node = {row["kind"] + ":" + posixpath.dirname(row["manifestPath"]): key for key, row in records.items()}
    pending, build_nodes = [], set()
    for identity in build:
        row = records[identity]
        pending.append(row["kind"] + ":" + posixpath.dirname(row["manifestPath"]))
        if row.get("owner"):
            pending.append("npm:" + posixpath.dirname(records["npm:" + row["owner"]]["manifestPath"]))
    while pending:
        key = pending.pop()
        if key in build_nodes:
            continue
        build_nodes.add(key)
        pending.extend(graph.get(key, {}).get("productDeps", set()))
        if key in by_node:
            dependencies.add(by_node[key])
    result["buildDependencyClosure"] = sorted(dependencies)
    result["buildNodes"] = sorted(build_nodes)
    result["sameVersionRebuildPolicy"] = "compare-original-archive-bytes-or-require-new-version"
    result["planHash"] = digest(result)
    return result


def main():
    parser = argparse.ArgumentParser(description="Calculate an independent chain's first-build fallback from exact Git objects")
    parser.add_argument("--chain", choices=sorted(CHAINS), required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    source = impact.revision(args.source)
    if impact.git("rev-parse", "HEAD").decode("ascii").strip() != source:
        raise ValueError("Checkout differs from the requested chain source")
    files, hashes = impact.snapshot(source)
    family = json.loads(files["docs/releases/v2-candidate.json"])
    plan = plan_chain(args.chain, {"sha": source, "tree": impact.git("rev-parse", source + "^{tree}").decode("ascii").strip()},
                      files, family, [], source_modes={name: row["mode"] for name, row in hashes.items()})
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
