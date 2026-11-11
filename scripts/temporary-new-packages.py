"""Temporary, manually dispatched publication of wholly new npm package names.

Consumes the independently signed original archives; never repacks, changes tags on
existing packages, or creates a GitHub release. Delete after the bootstrap.
"""
# cspell:words npmjs USERCONFIG GLOBALCONFIG readback
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


best = load('temporary_best_effort', 'release-npm-best-effort.py')
publisher = best.publisher
entry = best.entry
check = entry.check
WORKFLOW = '.github/workflows/temporary-new-packages.yaml'
PRODUCER_SOURCE = '74a09f11e189ac50bfe193885a910cbf264eefd8'
PRODUCER_RUN = 42964546726
PRODUCER_ATTEMPT = 1
CANDIDATE_ARTIFACT = 13834104358
CANDIDATE_DIGEST = 'sha256:567681d4a390754ab2391a036aab339d077bcfa0faae078a80df47318261c6ce'
PLATFORMS = ['darwin-arm64', 'darwin-x64', 'linux-arm64-gnu', 'linux-x64-gnu', 'win32-x64-msvc']
DOC_PLUGINS = ['docs', 'docs-seo', 'docs-snapshot', 'docs-snippets', 'ssr-css', 'syntax-highlight']
VUE_PLATFORMS = PLATFORMS + ['linux-arm64-musl', 'linux-x64-musl', 'win32-arm64-msvc', 'win32-ia32-msvc']
ALLOWLIST = ({'@farmfe/docs', '@farmfe/docs-theme-default'}
             | {'@farmfe/plugin-' + plugin + suffix for plugin in DOC_PLUGINS
                for suffix in [''] + ['-' + platform for platform in PLATFORMS]}
             | {'@farmfe/plugin-vue-jsx' + suffix for suffix in [''] + ['-' + platform for platform in VUE_PLATFORMS]})
assert len(ALLOWLIST) == 48


class OriginalProducerVerifier(entry.ReleaseVerifier):
    """A failed overall release may still have a successful signed producer."""
    def run(self, run_id, attempt, source_sha, current_run=None):
        check(type(run_id) is int and run_id > 0 and type(attempt) is int and attempt > 0,
              'Exact original run and attempt required')
        run = self.transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
        repository = self.transport.get('')
        check(run['id'] == run_id and run['run_attempt'] == attempt and run['status'] == 'completed'
              and run['path'] == entry.WORKFLOW and run['event'] == 'push'
              and run['head_branch'] == repository['default_branch'] == 'main'
              and run['head_sha'] == source_sha
              and run['repository']['id'] == repository['id'] == run['head_repository']['id'],
              'Original main producer identity differs or run has not completed')
        source = self.read_source(source_sha)
        route = '/branches/main'
        policy_sha = self.transport.get(route)['commit']['sha']
        comparison = self.transport.get('/compare/' + source_sha + '...' + policy_sha)
        check(comparison.get('status') in ['ahead', 'identical']
              and comparison.get('base_commit', {}).get('sha') == source_sha
              and comparison.get('merge_base_commit', {}).get('sha') == source_sha,
              'Producer source is not main history')
        jobs = entry.execution.jobs_for(self.transport, run)
        entry.evidence.successful_job(jobs, entry.PRODUCER)
        return run, repository, source, jobs, route, policy_sha


def verify_candidate(directory, environment):
    check(environment.get('GITHUB_ACTIONS') == 'true'
          and environment.get('GITHUB_REPOSITORY') == 'farm-fe/farm'
          and environment.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
          and environment.get('GITHUB_REF') == 'refs/heads/main'
          and environment.get('GITHUB_WORKFLOW_REF') == 'farm-fe/farm/' + WORKFLOW + '@refs/heads/main'
          and environment.get('GITHUB_SHA') == environment.get('GITHUB_WORKFLOW_SHA'),
          'Manual main workflow required')
    manifest = json.loads((directory / 'manifest.json').read_text(encoding='utf-8'))
    check(manifest.get('sourceSHA') == PRODUCER_SOURCE and manifest.get('producerRunId') == PRODUCER_RUN
          and manifest.get('producerAttempt') == PRODUCER_ATTEMPT, 'Candidate producer identity differs')
    best.new_report(manifest)
    transport = entry.evidence.transport_module.GitHubReadOnly(
        'farm-fe/farm', environment.get('GH_TOKEN', ''), environment=environment, allow_archive=True)
    source = entry.evidence.source_reader(PRODUCER_SOURCE)
    provenance = load('temporary_release_provenance', 'release-provenance.py')
    artifact, origin, _ = provenance.find_frozen_binding(transport, source, 'frozen', PRODUCER_RUN,
        PRODUCER_ATTEMPT, CANDIDATE_ARTIFACT, 'candidates')
    check(artifact.get('digest') == CANDIDATE_DIGEST and origin.get('run_attempt') == PRODUCER_ATTEMPT,
          'Signed original candidate artifact differs')
    proof = OriginalProducerVerifier(transport).verify(manifest, directory / 'archives', phase='prepublish')
    check(proof.get('prepublicationVerified') is True, 'Signed original archives not verified')
    selected = [row for row in manifest['packages'] if row['name'] in ALLOWLIST]
    check(len(selected) == 48 and {row['name'] for row in selected} == ALLOWLIST
          and all(row['version'] == '0.0.1' for row in selected), 'Fixed new-package inventory differs')
    return manifest


class NewPackageWriter:
    def __init__(self, token, directory):
        check(isinstance(token, str) and token and not any(c in token for c in '\0\r\n'),
              'Bypass token missing or invalid')
        self.token, self.directory = token, Path(directory)
        self.reader = entry.evidence.registry_module.RegistryReadOnly()
        self.scratch = tempfile.TemporaryDirectory(prefix='farm-new-npm-')
        home = Path(self.scratch.name)
        config, global_config = home / 'user.npmrc', home / 'global.npmrc'
        config.write_text('registry=https://registry.npmjs.org/\n//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}\n', encoding='utf-8')
        global_config.write_text('', encoding='utf-8')
        config.chmod(0o600); global_config.chmod(0o600)
        self.env = {'PATH': os.environ.get('PATH', os.defpath), 'HOME': str(home),
                    'NPM_CONFIG_USERCONFIG': str(config), 'NPM_CONFIG_GLOBALCONFIG': str(global_config),
                    'npm_config_cache': str(home / 'cache'), 'npm_config_ignore_scripts': 'true',
                    'npm_config_update_notifier': 'false'}
        probe = "import {resolveNpmCli} from './scripts/v2-npm-cli.mjs';console.log(JSON.stringify({node:process.execPath,cli:resolveNpmCli()}));"
        result = subprocess.run(['node', '--input-type=module', '-e', probe], env=self.env,
                                capture_output=True, text=True, timeout=15, check=True)
        selected = json.loads(result.stdout)
        self.node, self.cli = selected['node'], selected['cli']
        check(Path(self.node).is_absolute() and Path(self.cli).is_absolute(), 'Controlled npm missing')

    def close(self):
        self.token = None
        self.scratch.cleanup()

    def exists(self, row):
        try:
            data = self.reader.json('https://registry.npmjs.org/' + urllib.parse.quote(row['name'], safe=''))
            check(data.get('name') == row['name'], 'Registry package identity differs')
            return True
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return False
            raise ValueError('Registry package lookup failed') from None

    def publish(self, row, body):
        check(hashlib.sha256(body).hexdigest() == row['archiveSha256'], 'Archive bytes differ')
        metadata = publisher.npm_tarball.validate(row, body)
        check(not metadata.get('publishConfig', {}).get('provenance'), 'Archive requires provenance')
        with tempfile.TemporaryDirectory(prefix='sealed-', dir=self.scratch.name) as folder:
            archive = Path(folder) / 'package.tgz'
            with archive.open('xb') as stream:
                os.chmod(archive, 0o600); stream.write(body)
            try:
                result = subprocess.run([self.node, self.cli, 'publish', str(archive), '--ignore-scripts',
                                         '--access', 'public', '--tag', 'farm-release-0.0.1', '--provenance=false', '--registry=https://registry.npmjs.org'],
                                        env={**self.env, 'NODE_AUTH_TOKEN': self.token}, cwd=self.scratch.name,
                                        capture_output=True, timeout=120)
            except subprocess.TimeoutExpired as error:
                raise publisher.npm_failure('publish', row, timed_out=True, stdout=error.output, stderr=error.stderr) from None
            if result.returncode:
                raise publisher.npm_failure('publish', row, code=result.returncode,
                                            stdout=result.stdout, stderr=result.stderr)

    def verify(self, row):
        # Existing OIDC writer's read-only lookup verifies exact registry bytes.
        return publisher.RegistryWriter({'GITHUB_ACTIONS': 'true'}, self.directory).lookup(row) == row['archiveSha256']


def wait_for_registry(writer, row, sleep=time.sleep):
    """Read only: a successful npm response may precede public visibility."""
    for delay in [1, 2, 4, 8, 15]:
        if writer.verify(row):
            return True
        sleep(delay)
    return writer.verify(row)


def publish_new_packages(manifest, directory, writer, save):
    rows = [row for row in manifest['packages'] if row['name'] in ALLOWLIST]
    check(len(rows) == 48 and {row['name'] for row in rows} == ALLOWLIST
          and all(row['version'] == '0.0.1' for row in rows), 'Fixed new-package inventory differs')
    report = {'kind': 'temporary-new-package-publication', 'sourceSHA': manifest['sourceSHA'],
              'complete': False, 'success': False, 'packages': [
                  {'name': row['name'], 'version': row['version'], 'status': 'not-attempted',
                   'uploadAttempted': False, 'outcomeUncertain': False} for row in rows]}
    save(report)
    for row, result in zip(rows, report['packages']):
        stage = 'name-preflight'
        try:
            if writer.exists(row):
                stage = 'existing-byte-check'
                check(writer.verify(row), 'Existing package bytes not confirmed')
                result.update(status='already-published', registryBytesVerified=True)
            else:
                stage = 'archive-validation'
                body = best.read_archive(directory / 'archives', row)
                publisher.npm_tarball.validate(row, body)
                # A second name-level 404 limits races. npm also enforces immutable versions.
                stage = 'name-recheck'
                if writer.exists(row):
                    stage = 'existing-byte-check'
                    check(writer.verify(row), 'Existing package bytes not confirmed')
                    result.update(status='already-published', registryBytesVerified=True)
                else:
                    result.update(status='upload-started', uploadAttempted=True, outcomeUncertain=True)
                    save(report)
                    stage = 'publish'
                    writer.publish(row, body)
                    stage = 'registry-readback'
                    check(wait_for_registry(writer, row), 'Registry bytes not confirmed')
                    result.update(status='published', outcomeUncertain=False, registryBytesVerified=True)
        except Exception as error:
            result['status'] = 'failed'
            result['failedStage'] = stage
            result['error'] = str(error) if isinstance(error, publisher.NpmCommandFailure) else 'Operation failed; raw diagnostic withheld'
            # Do not retry an uncertain upload or misrepresent a successful readback as a successful call.
            if result['outcomeUncertain']:
                try:
                    if writer.verify(row): result.update(outcomeUncertain=False, registryBytesVerified=True)
                except Exception:
                    pass
        save(report)
    report['complete'] = True
    report['success'] = all(row['status'] in ['published', 'already-published'] for row in report['packages'])
    save(report)
    return report


def main():
    token = os.environ.pop('NPM_BYPASS_2FA_TOKEN', None)
    parser = argparse.ArgumentParser(); parser.add_argument('--candidate', required=True); parser.add_argument('--output', required=True)
    args = parser.parse_args(); directory = Path(args.candidate); output = Path(args.output)
    initial = {'kind': 'temporary-new-package-publication', 'complete': False, 'success': False,
               'packages': [{'name': name, 'version': '0.0.1', 'status': 'not-attempted',
                             'uploadAttempted': False} for name in sorted(ALLOWLIST)]}
    best.write_report(output, initial)
    try:
        check(os.environ.get('CONFIRM_PUBLICATION') == 'true', 'Explicit workflow confirmation required')
        check(isinstance(token, str) and token, 'Bypass token missing')
        manifest = verify_candidate(directory, os.environ)
        writer = NewPackageWriter(token, directory / 'archives')
        token = None
        try:
            with best.Deadline(seconds=65 * 60).guard():
                report = publish_new_packages(manifest, directory, writer, lambda value: best.write_report(output, value))
        finally:
            writer.close()
        return 0 if report['success'] else 1
    except (Exception, best.PublicationInterrupted, KeyboardInterrupt):
        if output.exists():
            report = json.loads(output.read_text(encoding='utf-8'))
            report['success'] = False
            report['error'] = 'Preparation or publication interrupted; inspect per-package status'
            best.write_report(output, report)
        print('Temporary publication failed; see the results artifact. Raw diagnostics withheld.')
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
