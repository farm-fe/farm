"""Trusted main-only release writer. Never invoked by PR validation.

Commands consume original release archives and keep a durable lock through
public consumer verification and exact tag/Release confirmation.
"""
# cspell:words argparse importlib npmjs urllib
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import urllib.error
import urllib.parse
import urllib.request


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    value = importlib.util.module_from_spec(spec); spec.loader.exec_module(value)
    return value


entry = load("publication_entry", "incremental-release-entry.py")
state = load("publication_state", "incremental-publication-state.py")
functional = load("publication_functional", "incremental-release-functional.py")
check = entry.check
ROLES = {"upload": "Publish exact independent release archives under durable lock",
         "finalize": "Finalize exact independent release and durable baseline"}


class GitHubWriter:
    def __init__(self, environment, stage, opener=None):
        check(stage in ROLES and environment.get("FARM_EXPLICIT_PUBLICATION") == "true", "Explicit publication authority required")
        # This guard also fixes the repository origin and rejects PR/local use.
        self.reader = entry.actions_transport(environment)
        check(environment.get("GITHUB_JOB") == ("publisher" if stage == "upload" else "finalize"), "Unexpected executing publication job")
        self.repository = environment["GITHUB_REPOSITORY"]
        self.prefix = "https://api.github.com/repos/" + self.repository
        self.token = environment["GH_TOKEN"]
        self.stage = stage
        self.opener = opener or urllib.request.build_opener(entry.evidence.transport_module.NoRedirect())

    def request(self, method, route, data=None, missing=False):
        allowed = [r"/git/ref/heads/farm-release-state", r"/git/commits/[a-f0-9]{40}", r"/git/trees/[a-f0-9]{40}",
                   r"/git/blobs/[a-f0-9]{40}", r"/git/blobs", r"/git/trees", r"/git/commits", r"/git/refs",
                   r"/git/refs/heads/farm-release-state"]
        if method == "GET":
            allowed += [r"/git/ref/tags/[A-Za-z0-9.-]+", r"/git/tags/[a-f0-9]{40}", r"/releases/tags/[A-Za-z0-9.-]+"]
        if self.stage == "finalize":
            allowed += [r"/git/ref/tags/[A-Za-z0-9.-]+", r"/git/tags/[a-f0-9]{40}", r"/releases", r"/releases/tags/[A-Za-z0-9.-]+"]
        check(method in ["GET", "POST", "PATCH"] and any(re.fullmatch(pattern, route) for pattern in allowed), "Unexpected publication API route")
        if method == "POST" and route == "/git/refs":
            target = (data or {}).get("ref", "")
            check(target == "refs/heads/farm-release-state" or self.stage == "finalize"
                  and re.fullmatch(r"refs/tags/(?:v\d+\.\d+\.\d+|packages-[a-f0-9]{16})", target), "Unexpected publication ref target")
        if method == "PATCH":
            check(route == "/git/refs/heads/farm-release-state" and (data or {}).get("force") is False, "Forced or unrelated publication update forbidden")
        check(not missing or method == "GET" and (route == "/git/ref/heads/farm-release-state"
              or route.startswith(("/git/ref/tags/", "/releases/tags/"))), "Unexpected missing-resource exception")
        body = None if data is None else json.dumps(data).encode()
        request = urllib.request.Request(self.prefix + route, data=body, method=method, headers={
            "Authorization": "Bearer " + self.token, "Accept": "application/vnd.github+json", "Content-Type": "application/json",
            "X-GitHub-Api-Version": "2022-11-28"})
        try:
            with self.opener.open(request, timeout=30) as response:
                raw = response.read(8 * 1024 * 1024 + 1)
        except urllib.error.HTTPError as error:
            if error.code == 404 and missing:
                return None
            raise ValueError("Publication API failed with HTTP " + str(error.code)) from None
        check(len(raw) <= 8 * 1024 * 1024, "Publication API response exceeds bound")
        return json.loads(raw.decode("utf-8"))


class RegistryWriter:
    def __init__(self, environment, directory, run_command=subprocess.run):
        self.environment, self.directory, self.command = environment, Path(directory), run_command
        self.reader = entry.evidence.registry_module.RegistryReadOnly(environment=environment)

    def lookup(self, row):
        name = urllib.parse.quote(row["name"], safe="")
        url = ("https://registry.npmjs.org/" + name + "/" + row["version"] if row["kind"] == "npm"
               else "https://crates.io/api/v1/crates/" + name + "/" + row["version"])
        try:
            metadata = self.reader.json(url)
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise ValueError("Registry lookup failed with HTTP " + str(error.code)) from None
            if row["kind"] == "npm":
                # Refuse a backfill rather than silently moving a newer latest.
                try:
                    latest = self.reader.json("https://registry.npmjs.org/" + name + "/latest")
                    check(re.fullmatch(r"\d+\.\d+\.\d+", latest.get("version", "")), "Stable latest version required")
                    check(tuple(map(int, latest["version"].split("."))) <= tuple(map(int, row["version"].split("."))), "Publishing would move latest backwards")
                except urllib.error.HTTPError as latest_error:
                    if latest_error.code != 404:
                        raise ValueError("Registry latest lookup failed") from None
            return None
        # Verify bytes against this response, with no second metadata fetch or
        # metadata-keyed cache that could attribute a B proof to metadata A.
        if row['kind'] == 'npm':
            check(metadata.get('name') == row['name'] and metadata.get('version') == row['version'], 'Registry npm identity differs')
            dist = metadata['dist']; url = dist['tarball']
            check(urllib.parse.urlsplit(url).hostname == 'registry.npmjs.org', 'Registry npm archive origin differs')
            actual = self.reader.archive(url)
            integrity = dist.get('integrity', '').split()
            check('sha512-' + actual['sha512'] in integrity if integrity else dist.get('shasum') == actual['sha1'],
                  'Registry npm archive differs from exact metadata')
        else:
            version = metadata['version']
            check(version.get('crate') == row['name'] and version.get('num') == row['version'], 'Registry Cargo identity differs')
            actual = self.reader.archive('https://static.crates.io/crates/' + row['name'] + '/' + row['name'] + '-' + row['version'] + '.crate')
            check(actual['sha256'] == version.get('checksum'), 'Registry Cargo archive differs from exact metadata')
        check(actual['sha256'] == row['archiveSha256'], 'Immutable registry version has different bytes')
        return actual['sha256']

    def publish(self, row, body):
        check(hashlib.sha256(body).hexdigest() == row["archiveSha256"], "Upload bytes differ from frozen archive")
        if row["kind"] == "cargo":
            token = self.environment.get("CARGO_REGISTRY_TOKEN")
            check(bool(token), "Existing Cargo publication credential missing")
            name = row.get("metadata")
            check(isinstance(name, str) and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,179}", name), "Original Cargo upload metadata required")
            metadata = (self.directory / name).read_bytes()
            check(len(metadata) <= 1024 * 1024 and hashlib.sha256(metadata).hexdigest() == row.get("metadataSha256"), "Cargo upload metadata differs")
            parsed = json.loads(metadata.decode("utf-8"))
            check(parsed.get("name") == row["name"] and parsed.get("vers") == row["version"], "Cargo upload identity differs")
            upload = struct.pack("<I", len(metadata)) + metadata + struct.pack("<I", len(body)) + body
            request = urllib.request.Request("https://crates.io/api/v1/crates/new", data=upload, method="PUT",
                headers={"Authorization": token, "Content-Type": "application/octet-stream", "User-Agent": "Farm-release-publisher"})
            try:
                with self.reader.opener.open(request, timeout=120) as response:
                    response.read(1024 * 1024 + 1)
            except urllib.error.HTTPError as error:
                raise ValueError("Cargo publication HTTP " + str(error.code)) from None
        else:
            token = self.environment.get("NPM_TOKEN")
            check(bool(token), "Existing npm publication credential missing")
            node = self.environment.get("FARM_NODE_EXECUTABLE", "node")
            probe = "import {resolveNpmCli} from './scripts/v2-npm-cli.mjs';console.log(resolveNpmCli());"
            resolved = self.command([node, "--input-type=module", "-e", probe], capture_output=True, text=True, timeout=15, check=True)
            cli = resolved.stdout.strip(); check(Path(cli).is_absolute() and Path(cli).is_file(), "Controlled npm CLI missing")
            scratch = Path(tempfile.mkdtemp(prefix="farm-release-npm-"))
            config = scratch / "user.npmrc"
            config.write_text("registry=https://registry.npmjs.org/\n//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}\n", encoding="utf-8")
            env = {key: value for key, value in self.environment.items() if key not in ["GH_TOKEN", "GITHUB_TOKEN", "CARGO_REGISTRY_TOKEN", "NPM_TOKEN"]}
            env.update({"NODE_AUTH_TOKEN": token, "NPM_CONFIG_USERCONFIG": str(config), "npm_config_cache": str(scratch / "cache")})
            result = self.command([node, cli, "publish", str((self.directory / row["archive"]).resolve()), "--ignore-scripts", "--access", "public",
                                   "--tag", "farm-release-" + row["version"], "--registry=https://registry.npmjs.org"], env=env, capture_output=True, timeout=120)
            check(result.returncode == 0, "npm publication failed; inspect independent registry state before resuming")

    def confirm(self, row):
        """Repair latest after an uncertain upload without republishing bytes."""
        if row['kind'] != 'npm':
            return
        def latest_version():
            try:
                latest = self.reader.json('https://registry.npmjs.org/' + urllib.parse.quote(row['name'], safe='') + '/latest')
            except urllib.error.HTTPError as error:
                if error.code == 404:
                    return None
                raise ValueError('Registry latest read failed') from None
            check(latest.get('name') == row['name'] and re.fullmatch(r'\d+\.\d+\.\d+', latest.get('version', '')), 'Registry latest identity differs')
            return tuple(map(int, latest['version'].split('.')))
        wanted = tuple(map(int, row['version'].split('.')))
        before = latest_version()
        if before is not None and before >= wanted:
            return
        token = self.environment.get('NPM_TOKEN'); check(bool(token), 'Existing npm publication credential missing')
        node = self.environment.get('FARM_NODE_EXECUTABLE', 'node')
        result = self.command([node, '--input-type=module', '-e', "import {resolveNpmCli} from './scripts/v2-npm-cli.mjs';console.log(resolveNpmCli());"],
                              capture_output=True, text=True, timeout=15, check=True)
        cli = result.stdout.strip(); check(Path(cli).is_absolute() and Path(cli).is_file(), 'Controlled npm CLI missing')
        scratch = Path(tempfile.mkdtemp(prefix='farm-release-npm-tag-')); config = scratch / 'user.npmrc'
        config.write_text('registry=https://registry.npmjs.org/\n//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}\n', encoding='utf-8')
        env = {key: value for key, value in self.environment.items() if key not in ['GH_TOKEN', 'GITHUB_TOKEN', 'CARGO_REGISTRY_TOKEN', 'NPM_TOKEN']}
        env.update({'NODE_AUTH_TOKEN': token, 'NPM_CONFIG_USERCONFIG': str(config), 'npm_config_cache': str(scratch / 'cache')})
        result = self.command([node, cli, 'dist-tag', 'add', row['name'] + '@' + row['version'], 'latest', '--registry=https://registry.npmjs.org'],
                              env=env, capture_output=True, timeout=120)
        check(result.returncode == 0, 'npm latest update failed; read back before retry')
        observed = latest_version()
        check(observed is not None and observed >= wanted, 'Registry latest read-back differs')


def inspect_release(api, manifest):
    tag = manifest.get("tag")
    check(isinstance(tag, str) and re.fullmatch(r"(?:v\d+\.\d+\.\d+|packages-[a-f0-9]{16})", tag), "Exact release tag required")
    marker = "<!-- farm-release-manifest:" + state.digest(manifest) + " -->"
    ref = api.request("GET", "/git/ref/tags/" + tag, missing=True)
    obj = ref["object"] if ref else None
    for _ in range(5):
        if obj and obj["type"] == "tag":
            obj = api.request("GET", "/git/tags/" + obj["sha"])["object"]
    check(obj is None or obj.get("type") == "commit" and obj.get("sha") == manifest["sourceSHA"], "Existing tag source differs")
    release = api.request("GET", "/releases/tags/" + tag, missing=True)
    check(release is None or obj is not None and release.get("draft") is False and release.get("prerelease") is False
          and marker in release.get("body", ""), "Existing Release provenance differs")
    return obj, release


def verify_release_checkpoint(api, manifest, release):
    checkpoint = entry.load('final_release_checkpoint', 'incremental-release-checkpoint.py')
    source = entry.evidence.source_reader(manifest['sourceSHA'])
    return checkpoint.decode(release, manifest, source, api.reader.get('')['id'])


def completed_publication_receipt(api, manifest, verifier, directory, current):
    proof = verifier.verify(manifest, directory / 'archives', phase='postpublish', current_run=current)
    _, release = inspect_release(api, manifest)
    check(proof.get('verified') is True and release is not None, 'Independent completed Release confirmation missing')
    verify_release_checkpoint(api, manifest, release)
    return True


def confirm_release(api, manifest, proof):
    tag = manifest["tag"]
    marker = "<!-- farm-release-manifest:" + state.digest(manifest) + " -->"
    obj, release = inspect_release(api, manifest)
    if obj is None:
        api.request("POST", "/git/refs", {"ref": "refs/tags/" + tag, "sha": manifest["sourceSHA"]})
    checkpoint = None
    if release is None:
        checkpoint = entry.release_checkpoint(manifest, proof)
        api.request("POST", "/releases", {"tag_name": tag, "name": "Farm " + tag, "body": marker + "\nExact source " + manifest["sourceSHA"] + "\n" + checkpoint, "draft": False, "prerelease": False})
    obj, release = inspect_release(api, manifest)
    check(obj is not None and release is not None, "Exact tag and Release read-back missing")
    if checkpoint is not None:
        check(checkpoint in release.get('body', ''), 'Persistent release checkpoint read-back differs')
    verify_release_checkpoint(api, manifest, release)
    return {"manifestHash": state.digest(manifest), "sourceSHA": manifest["sourceSHA"], "verified": True, "releaseId": release["id"]}


PUBLIC_PLATFORMS = ['ubuntu-24.04', 'macos-latest', 'windows-latest']
PUBLIC_UPLOAD = 'Upload independent public consumer report'


def report_binding(manifest, run_id, attempt, platform):
    return {'manifestHash': state.digest(manifest), 'sourceSHA': manifest['sourceSHA'], 'sourceTree': manifest['sourceTree'],
            'producerRunId': manifest['producerRunId'], 'producerAttempt': manifest['producerAttempt'],
            'consumerRunId': run_id, 'consumerAttempt': attempt, 'platform': platform}


def report_producer(transport, manifest, run_id, attempt, platform):
    check(platform in PUBLIC_PLATFORMS and type(attempt) is int and attempt >= manifest['producerAttempt'], 'Public report platform or attempt differs')
    run = transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
    repository = transport.get('')
    check(run['id'] == run_id == manifest['producerRunId'] and run['run_attempt'] == attempt and run['path'] == entry.WORKFLOW
          and run['head_sha'] == manifest['sourceSHA'] and run['head_branch'] == repository['default_branch'] == 'main'
          and run['event'] in ['push', 'workflow_dispatch'] and run['repository']['id'] == repository['id'] == run['head_repository']['id'],
          'Public consumer source or main origin differs')
    jobs = entry.execution.jobs_for(transport, run)
    name = 'Independent public registry consumer (' + platform + ')'
    matches = [job for job in jobs if job['name'] == name or job['name'].endswith(' / ' + name)]
    check(len(matches) == 1, 'Missing or ambiguous public consumer platform job')
    job = matches[0]
    check(job.get('run_id', run_id) == run_id and job.get('run_attempt', attempt) == attempt, 'Public consumer job belongs to another attempt')
    return job


def verify_public_report(transport, manifest, run_id, attempt, platform, artifact_id):
    job = report_producer(transport, manifest, run_id, attempt, platform)
    check(job['status'] == 'completed' and job['conclusion'] == 'success', 'Public consumer platform job did not succeed')
    artifact = transport.get('/actions/artifacts/' + str(artifact_id))
    name = 'release-public-' + platform + '-' + str(attempt)
    check(artifact['id'] == artifact_id and artifact['name'] == name and artifact['workflow_run']['id'] == run_id
          and artifact['expired'] is False, 'Public report artifact producer differs')
    entry.execution.upload_binding(job, PUBLIC_UPLOAD, artifact)
    report = functional.exact_plan(transport.archive(artifact_id), artifact, 'result.json')
    check(report.get('publicationBinding') == report_binding(manifest, run_id, attempt, platform), 'Public report frozen manifest or platform binding differs')
    check(report.get('status') == 'passed' and report.get('publicRegistry') is True and report.get('workspacePatches') is False
          and report.get('cleanupConfirmed') is True, 'Public consumer or confirmed cleanup failed')
    check({'npm-install', 'esm-cjs', 'types', 'docs-build', 'sdk'}.issubset(report.get('stages', [])), 'Public consumer stages missing')
    selected = {'platform': platform, 'runId': run_id, 'attempt': attempt, 'artifactId': artifact_id,
                'artifactDigest': artifact['digest'], 'jobId': job['id'], 'manifestHash': state.digest(manifest)}
    return report, selected


def verify_public_selection(transport, manifest, run_id, maximum_attempt, selections):
    check(isinstance(selections, list) and len(selections) == 3 and {row.get('platform') for row in selections} == set(PUBLIC_PLATFORMS),
          'Exactly one independent report per public platform required')
    verified = []
    for selected in selections:
        check(type(selected.get('attempt')) is int and selected['attempt'] <= maximum_attempt, 'Public report attempt is newer than its acceptance proof')
        _, actual = verify_public_report(transport, manifest, run_id, selected['attempt'], selected['platform'], selected['artifactId'])
        check(selected == actual, 'Public report provenance changed or was forged')
        verified.append(actual)
    return sorted(verified, key=lambda row: row['platform'])


def collect_public_reports(transport, manifest, current, directory):
    rows = []
    for page in range(1, 4):
        batch = transport.get('/actions/runs/' + str(current['runId']) + '/artifacts?per_page=100&page=' + str(page))['artifacts']
        rows.extend(batch)
        if len(batch) < 100:
            break
    else:
        raise ValueError('Public report artifact inventory exceeded bound')
    reports, selected = {}, []
    for platform in PUBLIC_PLATFORMS:
        pattern = r'release-public-' + re.escape(platform) + r'-([1-9][0-9]*)'
        candidates = []
        for row in rows:
            match = re.fullmatch(pattern, row['name'])
            if match and row.get('expired') is False and manifest['producerAttempt'] <= int(match[1]) <= current['attempt']:
                candidates.append((int(match[1]), row))
        check(len(candidates) <= 64 and len({attempt for attempt, row in candidates}) == len(candidates), 'Ambiguous or excessive public platform artifacts')
        for attempt, artifact in sorted(candidates, key=lambda row: row[0], reverse=True):
            job = report_producer(transport, manifest, current['runId'], attempt, platform)
            if job['status'] != 'completed' or job['conclusion'] != 'success':
                continue
            reports[platform], selection = verify_public_report(transport, manifest, current['runId'], attempt, platform, artifact['id'])
            selected.append(selection)
            break
        check(platform in reports, 'Successful public consumer report missing for ' + platform)
    target = Path(directory); target.mkdir(parents=True, exist_ok=False)
    for platform, report in reports.items():
        output = target / ('release-public-' + platform); output.mkdir()
        (output / 'result.json').write_text(json.dumps(report, sort_keys=True) + '\n', encoding='utf-8')
    selection = {'manifestHash': state.digest(manifest), 'reports': sorted(selected, key=lambda row: row['platform'])}
    (target / 'selection.json').write_text(json.dumps(selection, sort_keys=True) + '\n', encoding='utf-8')
    return selection


class RecoveryVerifier(entry.ReleaseVerifier):
    """Reuse the release seal protocol, retaining its original producer attempt.

    A failed workflow may contain a successful producer and candidate consumers.
    Only that same frozen run's later attempt may resume it, after independent
    terminal-attempt and original upload-step checks; CI runs never qualify.
    """
    def __init__(self, transport, current, read_source=entry.evidence.source_reader, frozen=None):
        super().__init__(transport, read_source)
        self.current, self.frozen = current, frozen
        self.candidate_selection = None

    def run(self, run_id, attempt, source_sha, current_run=None):
        if attempt == self.current['attempt'] and self.frozen is None:
            return self.candidate_jobs(super().run(run_id, attempt, source_sha, self.current))
        check(run_id == self.current['runId'] and type(attempt) is int and 0 < attempt <= self.current['attempt'], 'Recovery requires the original release run')
        run = self.transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
        repository = self.transport.get('')
        check(run['id'] == run_id and run['run_attempt'] == attempt and run['path'] == entry.WORKFLOW
              and run['repository']['id'] == repository['id'] == run['head_repository']['id']
              and run['event'] in ['push', 'workflow_dispatch'] and run['head_branch'] == repository['default_branch']
              and run['head_sha'] == source_sha and (run['status'] == 'completed' if attempt < self.current['attempt'] else run['status'] == 'in_progress'),
              'Original release attempt is not terminal or its source differs')
        source = self.read_source(source_sha)
        route = '/branches/' + urllib.parse.quote(repository['default_branch'], safe='')
        policy_sha = self.transport.get(route)['commit']['sha']
        if self.frozen is None:
            if source['policyHash'] != self.read_source(policy_sha)['policyHash']:
                comparison = self.transport.get('/compare/' + source['sha'] + '...' + policy_sha)
                check(comparison.get('status') in ['ahead', 'identical'] and comparison.get('base_commit', {}).get('sha') == source['sha']
                      and comparison.get('merge_base_commit', {}).get('sha') == source['sha'], 'Unlocked release source is not proven trusted main history')
        else:
            # A later main policy cannot revoke an already locked transaction.
            # The original main run, sealed source/tree and persisted authorization
            # are checked independently; an arbitrary historical PR cannot qualify.
            check(self.frozen['sourceSHA'] == source_sha and self.frozen['sourceTree'] == source['tree']
                  and self.frozen['policyHash'] == source['policyHash'] and self.frozen['producerRunId'] == run_id
                  and self.frozen['producerAttempt'] == attempt, 'Frozen release policy or original producer differs')
        jobs = entry.execution.jobs_for(self.transport, run)
        return self.candidate_jobs((run, repository, source, jobs, route, policy_sha))

    def candidate_jobs(self, facts):
        run, repository, source, jobs, route, policy_sha = facts
        # Use exactly the same content/attempt/artifact selector as the durable
        # baseline reader. Successful intermediate attempts must not disappear.
        reader_module = entry.load('publication_candidate_baseline', 'incremental-release-baseline.py')
        reader = reader_module.ReleaseBaselineReader(self.transport, None, self.read_source)
        selected = reader.candidates(self.verifying_manifest, repository, self.current['attempt'])
        observed = {}
        expected = {'ubuntu-24.04': ('linux', 'x64', 'linux-x64-gnu'),
                    'macos-latest': ('darwin', 'arm64', 'darwin-arm64'),
                    'windows-latest': ('win32', 'x64', 'win32-x64-msvc')}
        for platform in PUBLIC_PLATFORMS:
            choice = selected[platform]; attempt = choice['attempt']
            if attempt not in observed:
                candidate = self.transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt))
                observed[attempt] = entry.execution.jobs_for(self.transport, candidate)
            name = 'Independent release candidate consumer (' + platform + ')'
            job = entry.evidence.successful_job(observed[attempt], name)
            check(job.get('run_id') == run['id'] and job.get('run_attempt', attempt) == attempt, 'Candidate consumer job belongs to another attempt')
            artifact = self.transport.get('/actions/artifacts/' + str(choice['artifactId']))
            check(artifact.get('digest') == choice['artifactDigest'], 'Selected candidate artifact digest changed')
            report = functional.exact_plan(self.transport.archive(choice['artifactId']), artifact, 'result.json')
            os_name, arch, abi = expected[platform]
            check(report.get('platform') == {'os': os_name, 'arch': arch, 'abi': abi}, 'Candidate report OS or ABI differs')
            jobs = [old for old in jobs if old['name'] != name and not old['name'].endswith(' / ' + name)] + [job]
        self.candidate_selection = selected
        return run, repository, source, jobs, route, policy_sha

    def verify(self, manifest, archive_directory, phase='prepublish', current_run=None):
        self.verifying_manifest = manifest
        # All original source, plan, inventories, byte hashes, producer upload
        # and three candidate-consumer roles are rechecked by the base verifier.
        if self.frozen is not None:
            check(self.frozen['transaction'] == state.digest(manifest), 'Frozen authorization belongs to another manifest')
        preflight = super().verify(manifest, archive_directory, phase='prepublish', current_run=self.current)
        if phase == 'prepublish':
            return {**preflight, 'candidateReports': self.candidate_selection}
        check(phase in ['postpublish', 'complete'], 'Unknown publication verification phase')
        rows = []
        for page in range(1, 4):
            response = self.transport.get('/actions/runs/' + str(self.current['runId']) + '/artifacts?per_page=100&page=' + str(page))
            rows.extend(response['artifacts'])
            if len(response['artifacts']) < 100:
                break
        prefix = 'farm-release-public-acceptance-' + manifest['sourceSHA'] + '-'
        candidates = [row for row in rows if row.get('expired') is False and row['name'].startswith(prefix)
                      and row['name'][len(prefix):].isdigit()
                      and manifest['producerAttempt'] <= int(row['name'][len(prefix):]) <= self.current['attempt']]
        check(candidates, 'Original public acceptance artifact missing')
        selected = max(candidates, key=lambda row: (int(row['name'][len(prefix):]), row['id']))
        attempt = int(selected['name'][len(prefix):])
        run = self.transport.get('/actions/runs/' + str(self.current['runId']) + '/attempts/' + str(attempt))
        check(run['id'] == self.current['runId'] and run['run_attempt'] == attempt and run['path'] == entry.WORKFLOW
              and run['head_sha'] == manifest['sourceSHA'], 'Public acceptance run differs')
        job = entry.evidence.successful_job(entry.execution.jobs_for(self.transport, run), entry.CONSUMER)
        proof, artifact = entry.execution.fetch_json_artifact(self.transport, run, selected['name'])
        entry.execution.upload_binding(job, 'Upload independent release public acceptance', artifact)
        check(proof.get('manifestHash') == state.digest(manifest) and proof.get('sourceTree') == manifest['sourceTree']
              and proof.get('planHash') == manifest['planHash'] and proof.get('producerJobId') == job['id'], 'Public acceptance frozen identity differs')
        check(proof.get('packages') == {row['kind'] + ':' + row['name']: row['archiveSha256'] for row in manifest['packages']}
              and all(proof.get(key) is True for key in ['registryInstall', 'nativeABI', 'latestTargets']), 'Actual public acceptance is incomplete')
        reports = verify_public_selection(self.transport, manifest, self.current['runId'], attempt, proof.get('consumerReports'))
        actual = entry.evidence.registry_module.RegistryReadOnly().verify({row['kind'] + ':' + row['name']: row for row in manifest['packages']})
        check(actual == proof['packages'], 'Public registry changed after consumer acceptance')
        return {'manifestHash': state.digest(manifest), 'sourceSHA': manifest['sourceSHA'], 'sourceTree': manifest['sourceTree'],
                'producerRunId': manifest['producerRunId'], 'producerAttempt': manifest['producerAttempt'],
                'publicAttempt': attempt, 'consumerReports': reports, 'candidateReports': self.candidate_selection, 'verified': True, 'publicationExecuted': False}


def frozen_authorization(store, manifest, source, repository_id, current):
    """Load only the existing transaction claim; never trust a manifest's policy claim."""
    _, saved = store.read(); transaction = state.digest(manifest)
    active = saved.get('active')
    if active is not None:
        check(active.get('transaction') == transaction and active.get('manifest') == manifest,
              'Unfinished publication requires its original manifest')
        anchor = active.get('authorization')
    elif saved.get('baseline') == transaction:
        complete = saved.get('completed', {}).get(transaction)
        check(isinstance(complete, dict) and complete.get('manifest') == manifest, 'Completed frozen manifest differs')
        anchor = complete.get('authorization')
    else:
        return None
    check(isinstance(anchor, dict) and anchor.get('repositoryId') == repository_id and anchor.get('runId') == current['runId']
          and anchor.get('sourceSHA') == source['sha'] == manifest['sourceSHA']
          and anchor.get('sourceTree') == source['tree'] == manifest['sourceTree']
          and anchor.get('workflow') == entry.WORKFLOW and anchor.get('policyHash') == source['policyHash']
          and anchor.get('producerRunId') == manifest['producerRunId'] and anchor.get('producerAttempt') == manifest['producerAttempt']
          and type(anchor.get('attempt')) is int and manifest['producerAttempt'] <= anchor['attempt'] <= current['attempt'],
          'Persisted frozen authorization or source policy differs')
    return {**anchor, 'transaction': transaction}


def verify_locked_plan(transaction, source, transport, baseline_reader=entry.published_baseline, planner=entry.source_plan):
    """Independently re-read/recompute under the owned publication fence."""
    transaction.owned(); manifest = transaction.manifest
    def reject_if_proven_unwritten(reason):
        # Re-read ownership and per-package state; never infer no upload merely
        # from a local exception, an old snapshot or a failed network response.
        _, latest = transaction.owned()
        if latest['active']['phase'] == 'uploading' and not latest['active']['packages']:
            transaction.reject_unwritten(reason)
    try:
        baseline = baseline_reader(transport, manifest['baseline']) if manifest['baseline'] is not None else None
        fresh = planner(source, baseline)
        transaction.owned()
        matches = (fresh.get('sourceSHA') == manifest['sourceSHA'] and fresh.get('sourceTree') == manifest['sourceTree']
                   and fresh.get('planHash') == manifest['planHash'] and fresh.get('publicationBlocked') is False
                   and fresh.get('build') == manifest.get('rebuild') and fresh.get('versionRequired') == manifest.get('versionRequired')
                   and fresh.get('publish') == manifest.get('publish'))
    except Exception:
        reject_if_proven_unwritten('locked-plan-unavailable-before-registry-write')
        raise
    if not matches:
        reject_if_proven_unwritten('locked-plan-mismatch-before-registry-write')
        raise ValueError('Locked release plan changed; preserve uploaded originals or prepare a new candidate before any upload')
    return fresh


def terminal_owner(reader, current, source, owner):
    check(owner.get('runId') == current['runId'] and type(owner.get('attempt')) is int
          and 0 < owner['attempt'] < current['attempt'], 'Recovery owner is not an earlier attempt of this run')
    run = reader.get('/actions/runs/' + str(owner['runId']) + '/attempts/' + str(owner['attempt']))
    return (run['id'] == owner['runId'] and run['run_attempt'] == owner['attempt'] and run['status'] == 'completed'
            and run['path'] == entry.WORKFLOW and run['head_sha'] == source['sha']
            and run['event'] in ['push', 'workflow_dispatch'] and run['head_branch'] == 'main'
            and run['repository']['id'] == owner.get('repositoryId') == run['head_repository']['id'])


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("stage", choices=[*ROLES, "collect-public", "bind-public-proof"]); parser.add_argument("--candidate", required=True)
    parser.add_argument("--reports"); parser.add_argument("--proof")
    args = parser.parse_args(); directory = Path(args.candidate)
    manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    if args.stage in ['collect-public', 'bind-public-proof']:
        transport = entry.actions_transport()
        current = {'runId': int(os.environ['GITHUB_RUN_ID']), 'attempt': int(os.environ['GITHUB_RUN_ATTEMPT'])}
        check(os.environ.get('GITHUB_JOB') == 'public-proof' and manifest['sourceSHA'] == os.environ['GITHUB_SHA']
              and manifest['producerRunId'] == current['runId'], 'Read-only public proof role or frozen source differs')
        run = transport.get('/actions/runs/' + str(current['runId']) + '/attempts/' + str(current['attempt']))
        check(run['id'] == current['runId'] and run['run_attempt'] == current['attempt'] and run['path'] == entry.WORKFLOW
              and run['head_sha'] == manifest['sourceSHA'] and run['status'] == 'in_progress', 'Actual public proof execution differs')
        jobs = entry.execution.jobs_for(transport, run)
        check(len([job for job in jobs if job['name'] == entry.CONSUMER and job['status'] == 'in_progress']) == 1, 'Running public proof job missing')
        check(args.reports is not None, 'Public report directory required')
        if args.stage == 'collect-public':
            selection = collect_public_reports(transport, manifest, current, args.reports)
        else:
            check(args.proof is not None, 'Public proof ledger required')
            selection = json.loads((Path(args.reports) / 'selection.json').read_text(encoding='utf-8'))
            check(selection.get('manifestHash') == state.digest(manifest), 'Report selection belongs to another manifest')
            reports = verify_public_selection(transport, manifest, current['runId'], current['attempt'], selection.get('reports'))
            proof = json.loads(Path(args.proof).read_text(encoding='utf-8'))
            check(proof.get('manifestHash') == state.digest(manifest) and proof.get('sourceTree') == manifest['sourceTree']
                  and proof.get('planHash') == manifest['planHash'], 'Public proof frozen identity differs')
            proof['consumerReports'] = reports
            Path(args.proof).write_text(json.dumps(proof, sort_keys=True) + '\n', encoding='utf-8')
        print(json.dumps({'stage': args.stage, 'manifestHash': state.digest(manifest), 'publicationExecuted': False})); return
    api = GitHubWriter(os.environ, args.stage)
    source = entry.evidence.source_reader(os.environ["GITHUB_SHA"])
    check(manifest["sourceSHA"] == source["sha"] and manifest["sourceTree"] == source["tree"], "Writer checkout and frozen source differ")
    check(entry.chain.impact.git("rev-parse", "HEAD").decode("ascii").strip() == source["sha"], "Writer checkout differs from Actions source")
    run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
    check(manifest["producerRunId"] == run_id and type(manifest["producerAttempt"]) is int and 0 < manifest["producerAttempt"] <= attempt, "Writer must consume the original producer of this frozen release run")
    run = api.reader.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
    repository = api.reader.get("")
    check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"] and run["head_sha"] == source["sha"]
          and run["path"] == entry.WORKFLOW and run["status"] == "in_progress", "Trusted publication run differs")
    jobs = entry.execution.jobs_for(api.reader, run)
    check(len([job for job in jobs if job["name"] == ROLES[args.stage] and job["status"] == "in_progress"]) == 1, "Actual trusted write job missing")
    current = {"runId": run_id, "attempt": attempt}
    store = state.GitRefStore(state.GitHubStateAPI(api.request))
    frozen = frozen_authorization(store, manifest, source, repository["id"], current)
    verifier = RecoveryVerifier(api.reader, current, frozen=frozen)
    preflight = verifier.verify(manifest, directory / "archives", phase="prepublish", current_run=current)
    functional.verify_functional(api.reader, source, manifest.get("functionalEvidence"), current)
    owner = {"runId": run_id, "attempt": attempt, "repositoryId": repository["id"], "sourceSHA": source["sha"],
             "sourceTree": source["tree"], "workflow": entry.WORKFLOW, "policyHash": source["policyHash"],
             "producerRunId": manifest["producerRunId"], "producerAttempt": manifest["producerAttempt"]}
    transaction = state.Publication(store, manifest, owner,
                                    lambda value: preflight["manifestHash"] == state.digest(value) and preflight["prepublicationVerified"] is True)
    def completed_receipt(receipt):
        return completed_publication_receipt(api, manifest, verifier, directory, current)
    recover = lambda previous: terminal_owner(api.reader, current, source, previous)
    if args.stage == "upload":
        transaction.recover(recover, completed_receipt)
        if transaction.completed:
            print(json.dumps({'stage': args.stage, 'transaction': transaction.transaction, 'alreadyComplete': True})); return
        verify_locked_plan(transaction, source, api.reader)
        inspect_release(api, manifest)
        registry = RegistryWriter(os.environ, directory / "archives")
        transaction.upload(lambda row: (directory / "archives" / row["archive"]).read_bytes(), registry)
        transaction.handoff("public-acceptance")
    else:
        _, saved = transaction.store.read()
        active = saved.get('active')
        if active is not None and active.get('owner') == owner:
            transaction.claim_handoff("public-acceptance")
        else:
            transaction.recover(recover, completed_receipt)
        if transaction.completed:
            print(json.dumps({'stage': args.stage, 'transaction': transaction.transaction, 'alreadyComplete': True})); return
        verify_locked_plan(transaction, source, api.reader)
        transaction.finalize(lambda value: verifier.verify(value, directory / "archives", phase="postpublish", current_run=current),
                             lambda value, proof: confirm_release(api, value, proof))
    print(json.dumps({"stage": args.stage, "transaction": transaction.transaction}))


if __name__ == "__main__":
    main()
