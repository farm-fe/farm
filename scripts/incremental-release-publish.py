# cspell:words GLOBALCONFIG
"""Trusted main-only release writer. Never invoked by PR validation.

Commands consume original release archives and keep a durable lock through
public consumer verification and exact tag/Release confirmation.
"""
# cspell:words ENEEDAUTH EOTP EPUBLISHCONFLICT EPRIVATE EINVALIDPACKAGENAME EINVALIDTAGNAME EINTEGRITY EUSAGE ENOTFOUND ECONNRESET ETIMEDOUT
# cspell:words argparse importlib npmjs urllib USERCONFIG postpublish
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
consumer_bindings = load("publication_consumer_bindings", "release-consumer-bindings.py")
npm_tarball = load("publication_npm_tarball", "release-npm-tarball.py")
check = entry.check
ROLES = {"upload": "Publish exact independent release archives under durable lock",
         "finalize": "Finalize exact independent release and durable baseline"}


# Match only npm's code line; never forward arbitrary child text or debug logs.
NPM_ERRORS = {
    'E401': 'Registry authentication rejected',
    'ENEEDAUTH': 'Registry authentication required',
    'E403': 'Registry denied the operation; check package access and publication policy',
    'E404': 'Registry resource not found',
    'EOTP': 'Registry requires a one-time password',
    'EPUBLISHCONFLICT': 'Package version already exists',
    'EPRIVATE': 'Package is marked private',
    'EINVALIDPACKAGENAME': 'Invalid package name',
    'EINVALIDTAGNAME': 'Invalid distribution tag',
    'EINTEGRITY': 'Package integrity check failed',
    'EUSAGE': 'CLI usage rejected',
    'ENOTFOUND': 'Network name lookup failed',
    'EAI_AGAIN': 'Network name lookup temporarily failed',
    'ECONNRESET': 'Network connection reset',
    'ETIMEDOUT': 'Network operation timed out',
    'E429': 'Registry rate limit reached',
    'E500': 'Registry server error',
    'E502': 'Registry gateway error',
    'E503': 'Registry unavailable',
}


class NpmCommandFailure(ValueError):
    """Only bounded, allowlisted npm diagnostics may use this exception."""


class RegistryReadFailure(ValueError):
    """Preserve only bounded HTTP status for read-only retry decisions."""

    def __init__(self, operation, status):
        self.status = status if type(status) is int and 100 <= status <= 599 else None
        operation = operation if operation in ['latest', 'lookup'] else 'metadata'
        super().__init__('Registry ' + operation + ' read failed with HTTP '
                         + (str(self.status) if self.status is not None else 'unavailable'))


def npm_failure(operation, row, *, code=None, timed_out=False, stdout=None, stderr=None):
    if operation not in ['publish', 'dist-tag', 'resolve-cli', 'whoami']: operation = 'unknown'
    identity = 'controlled-cli'
    if row is not None:
        name, version = row.get('name'), row.get('version')
        valid = (isinstance(name, str) and len(name) <= 214
              and re.fullmatch(r'(?:@[a-z0-9._-]+/)?[a-z0-9._-]+', name)
              and isinstance(version, str) and re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version)
              and len(version) <= 60)
        identity = name + '@' + version if valid else 'invalid-identity'
    known = []
    for output in [stderr, stdout]:
        if not isinstance(output, (str, bytes)):
            continue
        text = output[-65536:]
        if len(output) > 65536:
            # The first retained line may start midway through arbitrary text.
            # Discard it rather than interpreting a sliced suffix as a code.
            newline = b'\n' if isinstance(text, bytes) else '\n'
            text = text.partition(newline)[2]
        if isinstance(text, bytes): text = text.decode('utf-8', errors='replace')
        # Same ANSI removal boundary as v2-release-diagnostics; unlike a log
        # tail, only exact allowlisted codes survive this parser.
        text = re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', text)
        for value in re.findall(r'^npm (?:ERR!|error) code ([A-Z][A-Z0-9_]{1,31})\r?$', text, re.M):
            if value in NPM_ERRORS and value not in known: known.append(value)
    returncode = code if type(code) is int and -(2 ** 31) <= code < 2 ** 32 else None
    status = 'timeout' if timed_out else 'exit=' + (str(returncode) if returncode is not None else 'unavailable')
    detail = '; '.join(value + ': ' + NPM_ERRORS[value] for value in known[:3]) or 'No recognized npm error code; raw output withheld'
    failure = NpmCommandFailure('npm operation=' + operation + ' package=' + identity + ' ' + status + '; ' + detail
                      + '; outcome uncertain: inspect independent registry state before resuming')
    failure.npm_codes = tuple(known[:3])
    failure.timed_out = timed_out is True
    failure.returncode = returncode
    return failure


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


def detach_oidc_credentials(environment):
    """Keep Actions OIDC secrets away from source and evidence subprocesses."""
    return {key: environment.pop(key) for key in
            ['ACTIONS_ID_TOKEN_REQUEST_URL', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN'] if key in environment}


def validate_npm_archives(manifest, directory):
    """Validate every frozen manifest before acquiring a publication transaction."""
    for row in manifest['packages']:
        check(row.get('kind') == 'npm', 'Only npm archives may enter OIDC publication')
        entry.strict_archive(directory, row)
        npm_tarball.validate(row, (Path(directory) / row['archive']).read_bytes())


class RegistryWriter:
    def __init__(self, environment, directory, run_command=subprocess.run):
        self.environment, self.directory, self.command = dict(environment), Path(directory), run_command
        self.reader = entry.evidence.registry_module.RegistryReadOnly(environment=environment)
        self.node = self.environment.get('FARM_NODE_EXECUTABLE', 'node')
        self._orders, self._npm, self._scratch = {}, None, None
        self._closed = False

    def __enter__(self):
        check(not self._closed, 'Registry writer is closed')
        return self

    def __exit__(self, *exc):
        self.close()

    def close(self):
        self._closed = True
        self._npm = None
        if self._scratch is not None:
            self._scratch.cleanup()
            self._scratch = None

    def npm_command(self, operation, row, argv, **options):
        try:
            result = self.command(argv, **options)
        except subprocess.TimeoutExpired as error:
            raise npm_failure(operation, row, timed_out=True, stdout=error.output, stderr=error.stderr) from None
        except subprocess.CalledProcessError as error:
            raise npm_failure(operation, row, code=error.returncode, stdout=error.stdout, stderr=error.stderr) from None
        except OSError:
            raise npm_failure(operation, row) from None
        if result.returncode != 0:
            raise npm_failure(operation, row, code=result.returncode,
                              stdout=getattr(result, 'stdout', None), stderr=getattr(result, 'stderr', None)) from None
        return result

    def npm(self):
        """Prepare the controlled CLI and token-free config once per writer."""
        check(not self._closed, 'Registry writer is closed')
        if self._npm is None:
            check(self.environment.get('GITHUB_ACTIONS') == 'true'
                  and self.environment.get('ACTIONS_ID_TOKEN_REQUEST_URL')
                  and self.environment.get('ACTIONS_ID_TOKEN_REQUEST_TOKEN'),
                  'GitHub Actions OIDC credentials missing')
            self._scratch = tempfile.TemporaryDirectory(prefix='farm-release-npm-')
            scratch = Path(self._scratch.name)
            try:
                config = scratch / 'user.npmrc'
                config.write_text('registry=https://registry.npmjs.org/\n', encoding='utf-8')
                config.chmod(0o600)
                global_config = scratch / 'global.npmrc'
                global_config.write_text('', encoding='utf-8')
                global_config.chmod(0o600)
                # Never pass ambient npm auth, NODE_OPTIONS, or npm configuration.
                # A fresh HOME, cwd and two config files prevent credential fallback.
                base = {'PATH': self.environment.get('PATH', os.environ.get('PATH', os.defpath)),
                        'HOME': str(scratch), 'NPM_CONFIG_USERCONFIG': str(config),
                        'NPM_CONFIG_GLOBALCONFIG': str(global_config),
                        'npm_config_cache': str(scratch / 'cache')}
                probe = "import {resolvePublisherNpmCli} from './scripts/v2-npm-cli.mjs';console.log(resolvePublisherNpmCli());"
                result = self.npm_command('resolve-cli', None, [self.node, '--input-type=module', '-e', probe],
                                      env=base, capture_output=True, text=True, timeout=15, check=True)
                cli = result.stdout.strip()
                check(Path(cli).is_absolute() and Path(cli).is_file(), 'Controlled npm CLI missing')
                # OIDC request credentials are available only to npm's publish/tag
                # subprocess, together with provenance metadata, not helper probes.
                permitted = ['CI', 'GITHUB_ACTIONS', 'GITHUB_SERVER_URL', 'GITHUB_REPOSITORY',
                             'GITHUB_REPOSITORY_ID', 'GITHUB_REPOSITORY_OWNER_ID', 'GITHUB_EVENT_NAME',
                             'GITHUB_REF', 'GITHUB_SHA', 'GITHUB_WORKFLOW_REF', 'GITHUB_RUN_ID',
                             'GITHUB_RUN_ATTEMPT', 'RUNNER_ENVIRONMENT',
                             'ACTIONS_ID_TOKEN_REQUEST_URL', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN']
                env = {**base, **{key: self.environment[key] for key in permitted if key in self.environment}}
                self._npm = (self.node, cli, env)
            except BaseException:
                self.close()
                raise
        return self._npm

    def latest_order(self, row):
        """Read latest afresh and compare using the verified bundled npm's SemVer."""
        try:
            latest = self.reader.json('https://registry.npmjs.org/' + urllib.parse.quote(row['name'], safe='') + '/latest')
            check(latest.get('name') == row['name'], 'Registry latest identity differs')
            version = latest.get('version')
            check(isinstance(version, str), 'Registry latest version missing')
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise RegistryReadFailure('latest', error.code) from None
            version = None
        target = row['version']
        check(isinstance(target, str), 'Invalid stable publication target')
        key = (version, target)
        # Cache only the pure, validated comparison under this fixed Node/code.
        # Registry identity and latest are always freshly fetched above.
        if key in self._orders:
            return self._orders[key]
        result = subprocess.run([self.node, str(Path(__file__).with_name('v2-npm-version-order.mjs')),
                                 json.dumps([version, target])], env={'PATH': self.environment.get('PATH', os.environ.get('PATH', os.defpath))},
                                capture_output=True, text=True, timeout=15)
        check(result.returncode == 0, 'Invalid registry SemVer or stable publication target')
        order = json.loads(result.stdout)
        check(order is None or type(order) is int and order in [-1, 0, 1], 'Invalid registry version comparison')
        if len(self._orders) < 4096:
            self._orders[key] = order
        return order

    def lookup(self, row):
        with self.reader.budget(600):
            return self._lookup(row)

    def _lookup(self, row):
        name = urllib.parse.quote(row["name"], safe="")
        url = ("https://registry.npmjs.org/" + name + "/" + row["version"] if row["kind"] == "npm"
               else "https://crates.io/api/v1/crates/" + name + "/" + row["version"])
        try:
            metadata = self.reader.json(url)
        except urllib.error.HTTPError as error:
            if error.code != 404:
                raise RegistryReadFailure('lookup', error.code) from None
            if row["kind"] == "npm":
                # A higher prerelease is also newer than the stable target.
                order = self.latest_order(row)
                check(order is None or order <= 0, "Publishing would move latest backwards")
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

    def validate_archive(self, row, body):
        npm_tarball.validate(row, body)

    def publish(self, row, body):
        check(row.get("kind") == "npm", "Cargo publication remains outside the new npm incremental publisher")
        check(hashlib.sha256(body).hexdigest() == row["archiveSha256"], "Upload bytes differ from frozen archive")
        self.validate_archive(row, body)
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
            node, cli, env = self.npm()
            name = row.get('archive')
            check(isinstance(name, str) and re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,179}', name),
                  'Archive must be a canonical basename')
            # npm opens a private byte-for-byte snapshot, never the path that
            # could have changed after our validated read. Do not repack it.
            with tempfile.TemporaryDirectory(prefix='sealed-', dir=env['HOME']) as directory:
                archive = Path(directory) / name
                with archive.open('xb') as stream:
                    os.chmod(archive, 0o600)
                    stream.write(body)
                self.npm_command("publish", row, [node, cli, "publish", str(archive), "--ignore-scripts", "--access", "public",
                                       "--tag", "farm-release-" + row["version"], "--registry=https://registry.npmjs.org", "--provenance"],
                                       env=env, cwd=env["HOME"], capture_output=True, timeout=120)

    def promote_latest(self, row):
        """Submit at most one latest update; visibility is a separate read."""
        if row['kind'] != 'npm':
            return 'retained'
        before = self.latest_order(row)
        if before is not None and before >= 0:
            return 'retained'
        node, cli, env = self.npm()
        # Recheck after CLI/config preparation. npm tags have no conditional CAS:
        # this closes that preparation window, not arbitrary external races.
        before = self.latest_order(row)
        if before is not None and before >= 0:
            return 'retained'
        self.npm_command('dist-tag', row, [node, cli, 'dist-tag', 'add', row['name'] + '@' + row['version'], 'latest', '--registry=https://registry.npmjs.org'],
                              env=env, cwd=env['HOME'], capture_output=True, timeout=120)
        return 'submitted'

    def latest_visible(self, row):
        """Fresh read-only proof that latest reached this version or a newer one."""
        if row['kind'] != 'npm':
            return True
        observed = self.latest_order(row)
        return observed is not None and observed >= 0

    def confirm(self, row):
        """Preserve synchronous confirmation for the durable publisher."""
        self.promote_latest(row)
        check(self.latest_visible(row), 'Registry latest read-back differs')


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


def verify_release_checkpoint(api, manifest, release, proof=None):
    checkpoint = entry.load('final_release_checkpoint', 'incremental-release-checkpoint.py')
    source = entry.evidence.source_reader(manifest['sourceSHA'])
    return checkpoint.decode(release, manifest, source, api.reader.get('')['id'], proof=proof)


def completed_publication_receipt(api, manifest, preflight, *, registry=None):
    # Only the completed-transaction no-op path may use durable publication
    # state. It never authorizes writes or substitutes for fresh public tests.
    # Waiting for this attempt's downstream public signer here creates a cycle.
    check(preflight.get('prepublicationVerified') is True
          and preflight.get('manifestHash')==state.digest(manifest)
          and preflight.get('sourceSHA')==manifest['sourceSHA']
          and preflight.get('sourceTree')==manifest['sourceTree'],
          'Already-complete confirmation requires current frozen-byte preflight')
    _, release = inspect_release(api, manifest)
    check(release is not None, 'Independent completed Release confirmation missing')
    checkpoint=verify_release_checkpoint(api, manifest, release)
    reader=entry.evidence.registry_module.RegistryReadOnly() if registry is None else registry
    expected={identity:row['archiveSha256'] for identity,row in checkpoint['packages'].items()}
    check(reader.verify(checkpoint['packages'])==expected,
          'Completed publication registry bytes are absent or changed')
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
        description = "Exact source " + manifest["sourceSHA"]
        if manifest.get('selectionPolicy') == 'registry-exact-version-v2':
            dispositions = proof['dispositions'].values()
            built = sum(row['originKind'] == 'source' for row in dispositions)
            retained = sum(row['originKind'] in ['registry', 'registry-race'] for row in dispositions)
            description += ("\nSource-built packages: " + str(built)
                            + "\nRegistry-retained packages: " + str(retained)
                            + "\nRetained versions reuse published artifacts, not rebuilt artifacts from this source. "
                              "This release does not claim current-source provenance for retained versions.")
        api.request("POST", "/releases", {"tag_name": tag, "name": "Farm " + tag, "body": marker + "\n" + description + "\n" + checkpoint, "draft": False, "prerelease": False})
    obj, release = inspect_release(api, manifest)
    check(obj is not None and release is not None, "Exact tag and Release read-back missing")
    if checkpoint is not None:
        check(checkpoint in release.get('body', ''), 'Persistent release checkpoint read-back differs')
    if manifest.get('selectionPolicy') == 'registry-exact-version-v2':
        verify_release_checkpoint(api, manifest, release, proof=proof)
    else:
        verify_release_checkpoint(api, manifest, release)
    return {"manifestHash": state.digest(manifest), "sourceSHA": manifest["sourceSHA"], "verified": True, "releaseId": release["id"]}


PUBLIC_PLATFORMS = ['ubuntu-24.04', 'macos-latest', 'windows-latest']
PUBLIC_UPLOAD = 'Upload independent public consumer report'


def report_binding(manifest, run_id, attempt, platform):
    return {'manifestHash': state.digest(manifest), 'sourceSHA': manifest['sourceSHA'], 'sourceTree': manifest['sourceTree'],
            'producerRunId': manifest['producerRunId'], 'producerAttempt': manifest['producerAttempt'],
            'consumerRunId': run_id, 'consumerAttempt': attempt, 'platform': platform}


def verify_public_report(transport, manifest, run_id, attempt, platform, artifact_id, *, trusted_output=None):
    check(isinstance(trusted_output,dict) and trusted_output.get('artifact_id')==str(artifact_id),
          'Independent fixed public upload binding required')
    run=transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(attempt))
    report,artifact,origin,job=consumer_bindings.receipt(transport,run,manifest,'public',platform,trusted_output)
    selected={'platform':platform,'runId':run_id,'attempt':origin['run_attempt'],'artifactId':artifact_id,
              'artifactDigest':artifact['digest'],'jobId':job['id'],'manifestHash':state.digest(manifest)}
    return report,selected


def verify_public_selection(transport, manifest, run_id, maximum_attempt, selections, *, upload_outputs=None):
    check(isinstance(selections,list) and len(selections)==3
          and {row.get('platform') for row in selections}==set(PUBLIC_PLATFORMS),
          'Exactly one independent report per public platform required')
    check(isinstance(upload_outputs,dict) and set(upload_outputs)==set(PUBLIC_PLATFORMS),
          'Fixed public consumer outputs required')
    verified=[]
    for selected in selections:
        platform=selected['platform'];output=upload_outputs[platform]
        check(type(selected.get('attempt')) is int and selected['attempt']<=maximum_attempt,
              'Public report attempt is newer than its acceptance proof')
        _,actual=verify_public_report(transport,manifest,run_id,maximum_attempt,platform,selected['artifactId'],trusted_output=output)
        check(selected==actual,'Public report provenance changed or was forged')
        verified.append(actual)
    return sorted(verified,key=lambda row:row['platform'])


def collect_public_reports(transport, manifest, current, directory, *, upload_outputs=None):
    check(isinstance(upload_outputs,dict) and set(upload_outputs)==set(PUBLIC_PLATFORMS),
          'Fixed public consumer outputs required')
    reports,selected={},[]
    for platform in PUBLIC_PLATFORMS:
        output=upload_outputs[platform]
        reports[platform],selection=verify_public_report(transport,manifest,current['runId'],current['attempt'],
            platform,int(output['artifact_id']),trusted_output=output)
        selected.append(selection)
    target=Path(directory);target.mkdir(parents=True,exist_ok=False)
    for platform,report in reports.items():
        output=target/('release-public-'+platform);output.mkdir()
        (output/'result.json').write_text(json.dumps(report,sort_keys=True)+'\n',encoding='utf-8')
    selection={'manifestHash':state.digest(manifest),'reports':sorted(selected,key=lambda row:row['platform'])}
    (target/'selection.json').write_text(json.dumps(selection,sort_keys=True)+'\n',encoding='utf-8')
    return selection


class RecoveryVerifier(entry.ReleaseVerifier):
    """Reuse the release seal protocol, retaining its original producer attempt.

    A failed workflow may contain a successful producer and candidate consumers.
    Only that same frozen run's later attempt may resume it, after independent
    terminal-attempt and original upload-step checks; CI runs never qualify.
    """
    def __init__(self, transport, current, read_source=entry.evidence.source_reader, frozen=None, *, receipt_provider=None, candidate_outputs=None, proof_provider=None):
        super().__init__(transport, read_source, receipt_provider=receipt_provider)
        self.candidate_outputs,self.proof_provider=candidate_outputs,proof_provider
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
        if entry.archive_integrity_mode(self.verifying_manifest):
            self.candidate_selection = {}
            return facts
        # Use exactly the same content/attempt/artifact selector as the durable
        # baseline reader. Successful intermediate attempts must not disappear.
        reader_module = entry.load('publication_candidate_baseline', 'incremental-release-baseline.py')
        reader = reader_module.ReleaseBaselineReader(self.transport, None, self.read_source, proof_provider=self.proof_provider)
        selected = reader.candidates(self.verifying_manifest, repository, self.current['attempt'], trusted_outputs=self.candidate_outputs)
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
        provenance=load('publication_signed_acceptance','release-provenance.py')
        run=self.transport.get('/actions/runs/'+str(self.current['runId'])+'/attempts/'+str(self.current['attempt']))
        source={'sha':manifest['sourceSHA'],'tree':manifest['sourceTree']}
        proof,artifact,origin,job,_,signed=provenance.signed_json(self.transport,run,source,entry.public_scope(manifest),'public',proof_provider=self.proof_provider)
        attempt=run['run_attempt']
        check(proof.get('manifestHash') == state.digest(manifest) and proof.get('sourceTree') == manifest['sourceTree']
              and proof.get('planHash') == manifest['planHash'] and proof.get('producerJobId') == job['id'], 'Public acceptance frozen identity differs')
        check(proof.get('packages') == {row['kind'] + ':' + row['name']: row['archiveSha256'] for row in manifest['packages']}
              and entry.public_integrity_verified(proof, manifest), 'Actual public acceptance is incomplete')
        reports = {}
        if entry.archive_integrity_mode(manifest):
            check(proof.get('kind')=='release-byte-integrity-v1', 'Registry proof kind differs')
        else:
            reports = verify_public_selection(self.transport, manifest, self.current['runId'], attempt, proof.get('consumerReports'),
                upload_outputs=provenance.consumer_outputs(signed,'public'))
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
        baseline = baseline_reader(transport, manifest['baseline']) if manifest['baseline'] is not None and manifest.get('registrySnapshot') is None else None
        if manifest.get('registrySnapshot') is not None:
            selector=entry.load('locked_registry_selection','release-registry-selection.py')
            registry = selector.Registry()
            selector.verify_retained(source,manifest['registrySnapshot'],lambda row: registry.lookup(row, verify_archive=True))
            fresh = planner(source,baseline,manifest['registrySnapshot'])
        else:
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
    oidc_credentials = detach_oidc_credentials(os.environ)
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
            selection = collect_public_reports(transport, manifest, current, args.reports,
                upload_outputs=consumer_bindings.outputs(os.environ['FARM_RELEASE_PUBLIC_UPLOADS']))
        else:
            check(args.proof is not None, 'Public proof ledger required')
            selection = json.loads((Path(args.reports) / 'selection.json').read_text(encoding='utf-8'))
            check(selection.get('manifestHash') == state.digest(manifest), 'Report selection belongs to another manifest')
            reports = verify_public_selection(transport, manifest, current['runId'], current['attempt'], selection.get('reports'),
                upload_outputs=consumer_bindings.outputs(os.environ['FARM_RELEASE_PUBLIC_UPLOADS']))
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
    candidate_outputs=consumer_bindings.outputs(os.environ['FARM_RELEASE_CANDIDATE_UPLOADS']) if args.stage=='upload' and not entry.archive_integrity_mode(manifest) else None
    verifier = RecoveryVerifier(api.reader, current, frozen=frozen, candidate_outputs=candidate_outputs)
    preflight = verifier.verify(manifest, directory / "archives", phase="prepublish", current_run=current)
    if args.stage == "upload":
        validate_npm_archives(manifest, directory / "archives")
    # PR quality is enforced by repository merge protection. Publication binds
    # the actual main source and frozen bytes above, not a separate test run.
    owner = {"runId": run_id, "attempt": attempt, "repositoryId": repository["id"], "sourceSHA": source["sha"],
             "sourceTree": source["tree"], "workflow": entry.WORKFLOW, "policyHash": source["policyHash"],
             "producerRunId": manifest["producerRunId"], "producerAttempt": manifest["producerAttempt"]}
    transaction = state.Publication(store, manifest, owner,
                                    lambda value: preflight["manifestHash"] == state.digest(value) and preflight["prepublicationVerified"] is True)
    def completed_receipt(receipt):
        return completed_publication_receipt(api, manifest, preflight)
    recover = lambda previous: terminal_owner(api.reader, current, source, previous)
    if args.stage == "upload":
        transaction.recover(recover, completed_receipt)
        if transaction.completed:
            print(json.dumps({'stage': args.stage, 'transaction': transaction.transaction, 'alreadyComplete': True})); return
        verify_locked_plan(transaction, source, api.reader)
        inspect_release(api, manifest)
        with RegistryWriter({**os.environ, **oidc_credentials}, directory / "archives") as registry:
            # Bound this stage below the 90-minute job cap; operation deadlines
            # do not inherit earlier source verification or idle reader time.
            with registry.reader.budget(4500):
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
