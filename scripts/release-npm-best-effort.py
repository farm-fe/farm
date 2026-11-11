"""Publish a complete source release independently, recording every outcome.

No durable lock or previous baseline is read or written here. Actions workflow
concurrency serializes writers. Full verified success may confirm the exact
tag and GitHub Release. Frozen producer proof and original bytes remain
mandatory; npm versions are never overwritten.
"""
# cspell:words argparse importlib readback postpublish setitimer SIGALRM SIGTERM ITIMER rpartition FINALIZER
import argparse
from contextlib import contextmanager
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


publisher = load('best_effort_transport', 'incremental-release-publish.py')
selector = load('best_effort_ranges', 'release-registry-selection.py')
entry = publisher.entry
check = entry.check
FINALIZER = 'Confirm fully verified npm release without durable state'


class PublicationInterrupted(BaseException):
    """Do not swallow the overall deadline inside a per-package exception."""


class Deadline:
    def __init__(self, seconds=4200, clock=time.monotonic):
        self.clock = clock
        self.end = clock() + seconds

    def remaining(self):
        remaining = self.end - self.clock()
        if remaining <= 0:
            raise PublicationInterrupted('Publication deadline exceeded')
        return remaining

    def command(self, argv, **options):
        options['timeout'] = min(options.get('timeout', 120), self.remaining())
        return subprocess.run(argv, **options)

    def registry_seconds(self):
        # RegistryReadOnly deliberately accepts bounded integer budgets only.
        return min(4500, math.ceil(self.remaining()))

    @contextmanager
    def guard(self):
        def interrupt(_signal, _frame):
            raise PublicationInterrupted('Publication deadline or runner interruption')
        seconds = self.remaining()
        previous = {kind: signal.signal(kind, interrupt) for kind in [signal.SIGALRM, signal.SIGTERM]}
        signal.setitimer(signal.ITIMER_REAL, seconds)
        try:
            yield
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
            for kind, handler in previous.items(): signal.signal(kind, handler)


def new_report(manifest):
    publisher.state.frozen_manifest(manifest)
    check(len(manifest['packages']) <= 1024 and manifest.get('publicationScope') == 'npm'
          and all(row.get('kind') == 'npm' for row in manifest['packages']), 'Only npm packages may be published')
    check(manifest.get('baseline') is None and manifest.get('registrySnapshot') is None,
          'Historical publication state cannot enter a new full release')
    family = manifest.get('family', [])
    expected = {row['kind'] + ':' + row['name'] for row in family}
    actual = {row['kind'] + ':' + row['name'] for row in manifest['packages']}
    check(len(expected) == len(family) and actual == expected == set(manifest['publish'])
          and set(manifest['rebuild']) == expected, 'Full source publication inventory required')
    return {'schemaVersion': 1, 'kind': 'npm-package-publication-results', 'stage': 'publisher',
            'manifestHash': entry.chain.digest(manifest), 'sourceSHA': manifest['sourceSHA'],
            'sourceTree': manifest['sourceTree'], 'producerRunId': manifest['producerRunId'],
            'producerAttempt': manifest['producerAttempt'], 'complete': False, 'success': False,
            'baselineAdvanced': False, 'releaseCreatedByThisStage': False,
            'packages': [{'name': row['name'], 'version': row['version'],
                          'archiveSha256': row['archiveSha256'], 'status': 'pending',
                          'uploadAttempted': False, 'outcomeUncertain': False,
                          'registryBytesVerified': False, 'errors': []}
                         for row in manifest['packages']]}


def diagnostic(stage, error):
    # Child output, HTTP bodies and exception strings may contain credentials.
    # Only the existing bounded allowlisted npm diagnostic is safe to retain.
    safe = {'Immutable registry version has different bytes': 'immutable-version-conflict',
            'Registry byte confirmation missing': 'registry-bytes-not-confirmed',
            'Frozen upload bytes changed': 'frozen-upload-bytes-changed',
            'GitHub Actions OIDC credentials missing': 'oidc-credentials-missing'}
    return {'stage': stage, 'code': safe.get(str(error), stage + '-failed'),
            'message': str(error) if isinstance(error, publisher.NpmCommandFailure)
            else stage + ' failed; raw diagnostic withheld'}


def required_dependencies(rows, metadata, *, peers_only=False):
    by_name = {row['name']: row for row in rows}
    result = {}
    for row in rows:
        name = row['name']
        if name not in metadata:
            continue
        package = metadata[name]
        dependencies = dict(package.get('dependencies', {}))
        # npm optional dependencies explicitly override dependencies. Optional
        # platform payload failures must not block otherwise usable platforms.
        for optional in package.get('optionalDependencies', {}):
            dependencies.pop(optional, None)
        peers = package.get('peerDependencies', {})
        peer_meta = package.get('peerDependenciesMeta', {})
        if peers_only:
            dependencies = {key: value for key, value in peers.items()
                            if peer_meta.get(key, {}).get('optional') is not True}
        required = set()
        for dependency, requirement in dependencies.items():
            check(isinstance(requirement, str), 'Invalid package dependency range')
            if requirement.startswith('npm:'):
                dependency, separator, requirement = requirement[4:].rpartition('@')
                check(separator and dependency, 'Invalid npm dependency alias')
            if dependency not in by_name:
                continue
            check(selector.stable_satisfies(by_name[dependency]['version'], requirement),
                  'Internal publication dependency incompatible')
            required.add(dependency)
        result[name] = required
    return result


def publish_packages(manifest, archive_reader, registry, report, save, *, remaining=lambda: 4200):
    rows = manifest['packages']; by_name = {row['name']: row for row in rows}
    outcomes = {row['name']: row for row in report['packages']}
    available = set()

    def failure(name, stage, error=None, blocked=None):
        outcome = outcomes[name]
        outcome['status'] = 'failed'
        outcome['errors'].append(diagnostic(stage, error) if error is not None
                                 else {'stage': stage, 'dependencies': sorted(blocked)})
        save(report)

    def lookup(row):
        remaining()
        actual = registry.lookup(row)
        check(actual is None or actual == row['archiveSha256'], 'Immutable registry version has different bytes')
        return actual is not None

    # Source, signatures and inventory are checked globally by the caller.
    # Validate all local inputs before npm runs; one bad package is isolated,
    # while its required consumers still need independent registry byte proof.
    metadata, dependencies, peers = {}, {}, {}
    for row in rows:
        name = row['name']
        try:
            remaining()
            body = archive_reader(row)
            check(len(body) == row['archiveBytes'], 'Frozen archive size differs')
            metadata[name] = publisher.npm_tarball.validate(row, body)
            dependencies.update(required_dependencies(rows, {name: metadata[name]}))
            peers.update(required_dependencies(rows, {name: metadata[name]}, peers_only=True))
        except Exception as error:
            metadata.pop(name, None)
            failure(name, 'archive-or-dependency-validation', error)

    def required_closure(name):
        seen = {name}; pending = list(dependencies.get(name, set()) | peers.get(name, set()))
        while pending:
            dependency = pending.pop()
            if dependency in seen: continue
            seen.add(dependency)
            pending.extend(dependencies.get(dependency, set()) | peers.get(dependency, set()))
        return seen - {name}

    closures = {name: required_closure(name) for name in metadata}
    # A whole-family publisher rechecks originals and dependency bytes several
    # times. Bound aggregate downloads by the authenticated inventory and exact
    # dependency edges rather than a fixed 3 GiB shared across all packages.
    # Individual archive (100 MiB), request and whole-job bounds still apply.
    reads = {row['name']: 4 for row in rows}
    for name in metadata:
        for dependency in dependencies[name]: reads[dependency] += 1
        for dependency in closures[name]: reads[dependency] += 1
    if hasattr(registry, 'reader'):
        registry.reader.total_byte_limit = min(1024 * 1024 ** 3, max(3 * 1024 ** 3,
            sum(row['archiveBytes'] * reads[row['name']] for row in rows)))
    # One package's conflict/read failure never prevents independent attempts.
    for row in rows:
        name = row['name']
        if outcomes[name]['status'] == 'failed':
            continue
        try:
            if lookup(row): available.add(name)
        except Exception as error:
            failure(name, 'registry-preflight', error)
    pending = [row['name'] for row in rows if outcomes[row['name']]['status'] == 'pending']
    while pending:
        progressed = False
        for name in pending[:]:
            # Only hard dependencies determine upload order. Peers constrain
            # completed public availability and may form valid cycles.
            if (dependencies[name] - available).intersection(pending):
                continue
            pending.remove(name); progressed = True
            row = by_name[name]; outcome = outcomes[name]
            blocked = []
            for dependency in sorted(dependencies[name]):
                try:
                    if not lookup(by_name[dependency]): blocked.append(dependency)
                except Exception:
                    blocked.append(dependency)
            if blocked:
                failure(name, 'required-dependency-unverified', blocked=blocked)
                continue
            try:
                exists = lookup(row)
            except Exception as error:
                failure(name, 'registry-before-upload', error)
                continue
            if not exists:
                try:
                    remaining()
                    body = archive_reader(row)
                    check(len(body) == row['archiveBytes']
                          and hashlib.sha256(body).hexdigest() == row['archiveSha256'], 'Frozen upload bytes changed')
                    publisher.npm_tarball.validate(row, body)
                    outcome['uploadAttempted'] = True
                    outcome['outcomeUncertain'] = True
                    outcome['status'] = 'upload-started'
                    save(report)
                    remaining()
                    registry.publish(row, body)
                except Exception as error:
                    # Never blindly retry an uncertain upload, including a
                    # timeout/auth error. Read immutable registry bytes first.
                    failure(name, 'publish', error)
            try:
                check(lookup(row), 'Registry byte confirmation missing')
                outcome['registryBytesVerified'] = True
                outcome['outcomeUncertain'] = False
                available.add(name)
            except Exception as error:
                available.discard(name)
                failure(name, 'registry-readback', error)
                continue
            if not outcome['errors']:
                outcome['status'] = 'registry-verified'
                outcome['outcome'] = 'already-published' if exists else 'published'
            save(report)
        if not progressed:
            for name in pending:
                failure(name, 'required-dependency-cycle', blocked=dependencies[name] - available)
            pending.clear()
    # Resolve required peers after all independent uploads. This supports the
    # Docs/theme peer cycle without promoting a package whose peer failed.
    for row in rows:
        name = row['name']; outcome = outcomes[name]
        if outcome['status'] != 'registry-verified':
            continue
        blocked = []
        for dependency in sorted(closures[name]):
            try:
                if dependency not in metadata or not lookup(by_name[dependency]): blocked.append(dependency)
            except Exception:
                blocked.append(dependency)
        if blocked:
            failure(name, 'required-dependency-unverified', blocked=blocked)
            continue
        try:
            remaining()
            registry.confirm(row)
            check(lookup(row), 'Registry byte confirmation missing')
            outcome['latestVerified'] = True
            outcome['status'] = 'success'
        except Exception as error:
            failure(name, 'latest-confirmation', error)
        save(report)
    report['complete'] = True
    report['success'] = all(row['status'] == 'success' for row in report['packages'])
    report['counts'] = {'total': len(rows), 'success': sum(row['status'] == 'success' for row in report['packages']),
                        'failed': sum(row['status'] == 'failed' for row in report['packages'])}
    save(report)
    return report


class SourceProofVerifier(publisher.RecoveryVerifier):
    def verify_archive(self, directory, row):
        # A signed manifest binds every expected byte digest. Retain global
        # source/plan/family/seal verification even if one local file is bad;
        # each actual upload independently requires its original exact bytes.
        try:
            entry.strict_archive(directory, row)
            return True
        except (ValueError, OSError):
            return False


def authorize(manifest, directory, environment, *, finalize=False):
    check(environment.get('FARM_EXPLICIT_PUBLICATION') == 'true'
          and environment.get('GITHUB_JOB') == ('finalize' if finalize else 'publisher'), 'Explicit publisher authority required')
    transport = entry.actions_transport(environment)
    source = entry.evidence.source_reader(environment['GITHUB_SHA'])
    check(source['sha'] == manifest['sourceSHA'] and source['tree'] == manifest['sourceTree']
          and entry.chain.impact.git('rev-parse', 'HEAD').decode('ascii').strip() == source['sha'],
          'Writer checkout and frozen source differ')
    current = {'runId': int(environment['GITHUB_RUN_ID']), 'attempt': int(environment['GITHUB_RUN_ATTEMPT'])}
    check(manifest['producerRunId'] == current['runId']
          and 0 < manifest['producerAttempt'] <= current['attempt'], 'Original same-run producer required')
    run = transport.get('/actions/runs/' + str(current['runId']) + '/attempts/' + str(current['attempt']))
    repository = transport.get('')
    check(run['id'] == current['runId'] and run['run_attempt'] == current['attempt']
          and run['repository']['id'] == repository['id'] == run['head_repository']['id']
          and run['head_sha'] == source['sha'] and run['path'] == entry.WORKFLOW
          and run['status'] == 'in_progress', 'Actual publication execution differs')
    jobs = entry.execution.jobs_for(transport, run)
    role = FINALIZER if finalize else entry.PUBLISHER
    check(len([job for job in jobs if job['name'] == role and job['status'] == 'in_progress']) == 1,
          'Actual publisher job missing')
    check(entry.archive_integrity_mode(manifest), 'New release requires source archive integrity policy')
    # RecoveryVerifier is only a producer proof adapter here: frozen=None, no
    # durable store, no transaction recovery, and no old baseline authorization.
    if finalize:
        entry.evidence.successful_job(jobs, entry.PUBLISHER)
        proof = publisher.RecoveryVerifier(transport, current).verify(manifest, directory / 'archives', phase='postpublish')
        check(proof.get('verified') is True, 'Complete signed public acceptance required')
    else:
        proof = SourceProofVerifier(transport, current).verify(manifest, directory / 'archives', phase='prepublish')
        check(proof.get('sourceAndSealVerified') is True, 'Independent source and producer seal proof missing')
    return proof


def finalize_release(manifest, directory, environment):
    new_report(manifest)  # Full independent source inventory, never old state.
    proof = authorize(manifest, directory, environment, finalize=True)
    return publisher.confirm_release(publisher.GitHubWriter(environment, 'finalize'), manifest, proof)


def read_archive(directory, row):
    entry.strict_archive(directory, row)
    return (directory / row['archive']).read_bytes()


def write_report(path, report):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    temporary.replace(path)


def main():
    oidc = publisher.detach_oidc_credentials(os.environ)
    parser = argparse.ArgumentParser()
    parser.add_argument('--candidate', required=True)
    parser.add_argument('--output')
    parser.add_argument('--finalize', action='store_true')
    args = parser.parse_args()
    directory = Path(args.candidate)
    manifest = json.loads((directory / 'manifest.json').read_text(encoding='utf-8'))
    if args.finalize:
        print(json.dumps(finalize_release(manifest, directory, os.environ)))
        return 0
    check(args.output is not None, 'Per-package result output required')
    output = Path(args.output)
    report = new_report(manifest)
    report.update(runId=int(os.environ['GITHUB_RUN_ID']), attempt=int(os.environ['GITHUB_RUN_ATTEMPT']))
    save = lambda value: write_report(output, value)
    save(report)  # No previous result can authorize or imply this attempt's success.
    try:
        # The first workflow step sets a 75-minute job-relative cutoff, leaving
        # 15 minutes for cleanup and artifact upload under the 90-minute cap.
        deadline = Deadline()
        if os.environ.get('FARM_PUBLICATION_DEADLINE'):
            deadline.end = min(deadline.end, float(os.environ['FARM_PUBLICATION_DEADLINE']))
        with deadline.guard():
            authorize(manifest, directory, os.environ)
            with publisher.RegistryWriter({**os.environ, **oidc}, directory / 'archives', run_command=deadline.command) as registry:
                with registry.reader.budget(deadline.registry_seconds()):
                    publish_packages(manifest, lambda row: read_archive(directory / 'archives', row), registry, report, save,
                                     remaining=deadline.remaining)
    except (Exception, PublicationInterrupted, KeyboardInterrupt) as error:
        report['success'] = False
        report['fatal'] = diagnostic('release-preflight-or-interruption', error)
        for row in report['packages']:
            if row['status'] not in ['success', 'failed']:
                row['status'] = 'failed'
                row['errors'].append({'stage': 'release-aborted-before-completion'})
        report['counts'] = {'total': len(report['packages']),
                            'success': sum(row['status'] == 'success' for row in report['packages']),
                            'failed': sum(row['status'] == 'failed' for row in report['packages'])}
        save(report)
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        outcomes = report['packages']
        with Path(os.environ['GITHUB_STEP_SUMMARY']).open('a', encoding='utf-8') as stream:
            stream.write('Npm publication: ' + str(sum(row['status'] == 'success' for row in outcomes))
                         + '/' + str(len(outcomes)) + ' packages succeeded.\n\n')
            for row in outcomes:
                if row['status'] != 'success':
                    stream.write('- ' + row['name'] + '@' + row['version'] + ': ' + row['status'] + '\n')
            stream.write('\nFull per-package results are in the publication-results artifact. No release baseline was advanced.\n')
    print(json.dumps({'complete': report['complete'], 'success': report['success'], 'baselineAdvanced': False}))
    return 0 if report['complete'] and report['success'] else 1


if __name__ == '__main__':
    sys.exit(main())
