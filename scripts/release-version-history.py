"""Read-only duplicate-submission guard; never restore old release authority.

A current exact metadata 200 always wins. A current 404 is insufficient after a
prior accepted or uncertain submission. No age or later rejection clears it.
Only authenticated Actions observations are used, without durable state/locks.
"""
# cspell:words importlib IFMT IFREG readback lookback SIGALRM ITIMER setitimer getitimer ENEEDAUTH EOTP EPRIVATE EINVALIDPACKAGENAME EINVALIDTAGNAME EUSAGE
from contextlib import contextmanager
import copy
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import signal
import stat
import time
import urllib.parse
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


selector = load('version_history_selector', 'release-registry-selection.py')
execution = load('version_history_execution', 'ci-execution-evidence.py')
evidence = execution.evidence
check = evidence.check
INCEPTION = '74a09f11e189ac50bfe193885a910cbf264eefd8'
# Reviewed UTC date of the first main push (run 42964546726), verified once
# against GitHub API metadata. Retaining that run object is not a runtime gate.
INCEPTION_DATE = '2026-11-11'
WORKFLOW = '.github/workflows/incremental-release.yaml'
PUBLISHER = 'Publish original npm archives and record every result'
PUBLISH_STEP = 'Verify producer proof and attempt every independent package'
UPLOAD_STEP = 'Upload all package outcomes even when publication fails'
MAX_PAGES = 10
MAX_ATTEMPTS = 20
MAX_EXECUTIONS = 200
MAX_BYTES = 4 * 1024 * 1024
REJECTIONS = {'ENEEDAUTH', 'E401', 'E403', 'EOTP', 'EPRIVATE', 'EINVALIDPACKAGENAME', 'EINVALIDTAGNAME', 'EUSAGE'}


def integer(value):
    return type(value) is int and 0 < value < 2 ** 63


def possible_submission(row):
    """A later rejection never overrides an earlier possible submission.

    Old reports marked even definite npm rejections outcomeUncertain. Recognize
    only the exact reviewed legacy ENEEDAUTH diagnostic, not arbitrary substrings.
    Structured records separate publish from dist-tag and record timeout state.
    """
    attempted = row.get('uploadAttempted')
    outcome = row.get('publishOutcome')
    accepted = outcome.get('accepted') if isinstance(outcome, dict) else row.get('accepted')
    if (accepted is True or row.get('accepted') is True or row.get('publishCommandSucceeded') is True
            or row.get('registryBytesVerified') is True or row.get('status') == 'success'):
        return True
    if attempted is False:
        return (row.get('outcomeUncertain') not in [None, False]
                or accepted not in [None, False] or outcome is not None
                or row.get('publishCommandCompletedAt') is not None
                or any(isinstance(error, dict) and error.get('stage') == 'publish' for error in row.get('errors', [])))
    if attempted is not True:
        return True
    if outcome is not None:
        if not (isinstance(outcome, dict) and set(outcome) == {'operation', 'returncode', 'timedOut', 'npmCodes', 'definiteRejection', 'accepted'}
                and outcome.get('operation') == 'publish' and type(outcome.get('returncode')) is int
                and 0 < outcome['returncode'] <= 255 and outcome.get('timedOut') is False
                and outcome.get('definiteRejection') is True and accepted is False
                and row.get('outcomeUncertain') is False):
            return True
        codes = outcome.get('npmCodes')
        return not (isinstance(codes, list) and bool(codes) and len(codes) <= 8
                    and len(set(codes)) == len(codes) and all(isinstance(code, str) and code in REJECTIONS for code in codes))
    if accepted is not None or row.get('registryBytesVerified') is not False or row.get('status') != 'failed':
        return True
    errors = row.get('errors')
    if not isinstance(errors, list) or not errors or not all(isinstance(error, dict) for error in errors):
        return True
    publishes = [error for error in errors if error.get('stage') == 'publish']
    expected = ('npm operation=publish package=' + row['name'] + '@' + row['version']
                + ' exit=1; ENEEDAUTH: Registry authentication required; outcome uncertain: inspect independent registry state before resuming')
    if not (len(publishes) == 1 and publishes[0] == {'stage': 'publish', 'code': 'publish-failed', 'message': expected}):
        return True
    # No timeout, alternate publish, successful/read-back/latest contradiction.
    return any(error is not publishes[0] and error != {
        'stage': 'registry-readback', 'code': 'registry-bytes-not-confirmed',
        'message': 'registry-readback failed; raw diagnostic withheld'} for error in errors)


def decode_report(body, artifact):
    check(0 < len(body) <= MAX_BYTES and len(body) == artifact.get('size_in_bytes')
          and artifact.get('digest') == 'sha256:' + hashlib.sha256(body).hexdigest(), 'History artifact digest differs')
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries = archive.infolist()
        check(len(entries) == 1 and entries[0].filename == 'results.json'
              and 0 < entries[0].file_size <= MAX_BYTES and not entries[0].is_dir()
              and stat.S_IFMT(entries[0].external_attr >> 16) in [0, stat.S_IFREG]
              and not entries[0].flag_bits & 1, 'History artifact members differ')
        data = archive.read(entries[0])
        check(len(data) == entries[0].file_size, 'History artifact member size differs')
    def unique_object(pairs):
        value = {}
        for key, row in pairs:
            check(key not in value, 'History report duplicate JSON key')
            value[key] = row
        return value
    return json.loads(data.decode('utf-8'), object_pairs_hook=unique_object)


def report_for(transport, run, job, source, family):
    name = 'npm-publication-results-' + str(run['id']) + '-' + str(run['run_attempt'])
    listing = transport.get('/actions/runs/' + str(run['id']) + '/artifacts?per_page=2&name=' + urllib.parse.quote(name, safe=''))
    rows = listing['artifacts']
    check(listing.get('total_count') == len(rows) == 1 and rows[0].get('name') == name, 'History artifact missing or ambiguous')
    artifact = transport.get('/actions/artifacts/' + str(rows[0]['id']))
    check(artifact.get('id') == rows[0]['id'] and artifact.get('name') == name and artifact.get('expired') is False
          and artifact.get('workflow_run', {}).get('id') == run['id']
          and 0 < artifact.get('size_in_bytes', 0) <= MAX_BYTES, 'History artifact origin differs')
    for key in ['head_sha', 'repository_id', 'head_repository_id']:
        expected = run['head_sha'] if key == 'head_sha' else run['repository']['id']
        if key in artifact['workflow_run']:
            check(artifact['workflow_run'][key] == expected, 'History artifact source differs')
    execution.upload_binding(job, UPLOAD_STEP, artifact)
    report = decode_report(transport.archive(artifact['id']), artifact)
    check(isinstance(report, dict) and type(report.get('schemaVersion')) is int and report['schemaVersion'] in [1, 2]
          and report.get('kind') == 'npm-package-publication-results' and report.get('stage') == 'publisher', 'History report policy differs')
    if report['schemaVersion'] == 2:
        check(report.get('selectionPolicy') == 'registry-exact-version-v2', 'History presence report policy differs')
    check(report.get('runId') == run['id'] and report.get('attempt') == run['run_attempt']
          and report.get('sourceSHA') == source['sha'] and report.get('sourceTree') == source['tree']
          and report.get('producerRunId') == run['id'] and integer(report.get('producerAttempt'))
          and report['producerAttempt'] <= run['run_attempt'], 'History report execution differs')
    evidence.chain_plan.exact(report.get('manifestHash'), 64)
    records = report.get('packages')
    check(isinstance(records, list) and len(records) == len(family) <= 1024
          and all(isinstance(row, dict) for row in records), 'History report family missing')
    identities = {('npm:' + row.get('name', '')): row for row in records}
    check(len(identities) == len(records) and set(identities) == set(family)
          and all(row.get('version') == family[key]['version'] for key, row in identities.items()), 'History report family differs')
    return identities, artifact


def verify_run(run, repository, run_id=None, attempt=None):
    check(isinstance(run, dict) and integer(run.get('id'))
          and (run_id is None or run['id'] == run_id)
          and integer(run.get('run_attempt')) and (attempt is None or run['run_attempt'] == attempt)
          and run.get('repository', {}).get('id') == repository['id'] == run.get('head_repository', {}).get('id')
          and run.get('path') == WORKFLOW and run.get('head_branch') == repository['default_branch']
          and run.get('event') in ['push', 'workflow_dispatch'], 'History run origin differs')
    evidence.chain_plan.exact(run.get('head_sha'), 40)


def inception_relation(transport, sha):
    if sha == INCEPTION:
        return True
    comparison = transport.get('/compare/' + INCEPTION + '...' + sha)
    check(comparison.get('base_commit', {}).get('sha') == INCEPTION, 'History inception comparison differs')
    if comparison.get('status') in ['ahead', 'identical'] and comparison.get('merge_base_commit', {}).get('sha') == INCEPTION:
        return True
    if comparison.get('status') == 'behind' and comparison.get('merge_base_commit', {}).get('sha') == sha:
        return False
    raise ValueError('History source inception ancestry unknown')


def publisher_job(transport, run):
    jobs = execution.jobs_for(transport, run)
    rows = [job for job in jobs if job.get('name') == PUBLISHER]
    check(len(rows) <= 1, 'History publisher ambiguous')
    if not rows:
        check(run.get('status') == 'completed', 'History publisher execution still pending')
        return None
    job = rows[0]
    check(integer(job.get('id')) and job.get('run_id') == run['id'] and job.get('run_attempt') == run['run_attempt']
          and job.get('head_sha') == run['head_sha'], 'History publisher identity differs')
    if job.get('status') == 'completed' and job.get('conclusion') == 'skipped':
        return None
    steps = [step for step in job.get('steps', []) if step.get('name') == PUBLISH_STEP]
    check(len(steps) == 1, 'History publish step missing or ambiguous')
    if steps[0].get('status') == 'completed' and steps[0].get('conclusion') == 'skipped':
        return None
    # A failed/cancelled run is not proof that npm never accepted a submission.
    return job


class HistoryDeadline(Exception):
    pass


class BoundedTransport:
    def __init__(self, transport, clock, seconds):
        self.transport, self.clock = transport, clock
        self.deadline = clock() + seconds

    def remaining(self):
        if self.clock() >= self.deadline:
            raise HistoryDeadline('History read deadline exceeded')

    def get(self, route):
        self.remaining()
        result = self.transport.get(route)
        self.remaining()
        return result

    def archive(self, artifact_id):
        self.remaining()
        result = self.transport.archive(artifact_id)
        self.remaining()
        return result


def run_inventory(transport, inception_created):
    """Split crowded fixed-date history instead of letting old runs eat a cap."""
    prefix = '/actions/workflows/' + urllib.parse.quote(Path(WORKFLOW).name, safe='') + '/runs?branch=main&created='
    requests, seen = 0, set()
    def query(window, page):
        nonlocal requests
        requests += 1
        check(requests <= 40, 'History partition read bound reached')
        listing = transport.get(prefix + urllib.parse.quote(window, safe='') + '&per_page=100&page=' + str(page))
        rows = listing['workflow_runs']
        total = listing.get('total_count')
        check(isinstance(rows, list) and len(rows) <= 100 and type(total) is int and total >= 0
              and len(rows) == min(100, max(0, total - 100 * (page - 1))), 'History run inventory invalid')
        return rows, total
    def page_window(window, first, total):
        rows = list(first)
        for page in range(2, (total + 99) // 100 + 1):
            next_rows, next_total = query(window, page)
            check(next_total == total, 'History run inventory changed while paging')
            rows.extend(next_rows)
        check(len(rows) == total, 'History run inventory incomplete')
        for row in rows:
            check(isinstance(row, dict) and integer(row.get('id')) and row['id'] not in seen, 'History run inventory duplicated')
            seen.add(row['id'])
        return rows
    def timestamp(value):
        check(isinstance(value, str) and re.fullmatch(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z', value), 'History run creation timestamp invalid')
        return datetime.fromisoformat(value.replace('Z', '+00:00'))
    window = '>=' + inception_created[:10]
    first, expected = query(window, 1)
    if expected <= MAX_PAGES * 100:
        return page_window(window, first, expected)
    start = datetime.fromisoformat(inception_created[:10]).replace(tzinfo=timezone.utc)
    end = max(timestamp(row.get('created_at')) for row in first)
    check(end >= start, 'History partition boundary differs')
    pending, result = [(start, end)], []
    while pending:
        begin, finish = pending.pop()
        interval = begin.strftime('%Y-%m-%dT%H:%M:%SZ') + '..' + finish.strftime('%Y-%m-%dT%H:%M:%SZ')
        rows, total = query(interval, 1)
        if total > MAX_PAGES * 100:
            seconds = int((finish - begin).total_seconds())
            check(seconds > 0, 'History one-second partition exceeds bound')
            middle = begin + timedelta(seconds=seconds // 2)
            pending.extend([(middle + timedelta(seconds=1), finish), (begin, middle)])
        else:
            result.extend(page_window(interval, rows, total))
    check(len(result) == expected, 'History partition coverage changed or is incomplete')
    return result


def _guard(source, snapshot, transport, *, current=None, read_source=evidence.source_reader, clock=time.monotonic, seconds=180):
    family = selector.validate(source, 'npm', snapshot)
    check(snapshot['schemaVersion'] == 2, 'History guard is only for stateless version presence')
    result = copy.deepcopy(snapshot)
    missing = {key for key, row in snapshot['packages'].items() if row['status'] == 'missing'}
    if not missing:
        return result
    def block(keys, reason, prior=None):
        for key in sorted(keys & missing):
            # A concrete earlier uncertain submission is stronger than an
            # unrelated incomplete-history diagnostic. Neither can be cleared.
            existing = result['packages'][key]
            if existing.get('reason') == 'prior-submission-unconfirmed':
                continue
            row = family[key]
            result['packages'][key] = {'status': 'unknown', 'name': row['name'], 'version': row['version'], 'reason': reason}
            if prior is not None:
                result['packages'][key]['priorSubmission'] = {**prior, 'name': row['name'], 'version': row['version']}
    transport = BoundedTransport(transport, clock, seconds)
    try:
        repository = transport.get('')
        check(integer(repository.get('id')) and repository.get('default_branch') == 'main', 'History repository policy differs')
        check(inception_relation(transport, source['sha']), 'Current source predates stateless publication')
        # Fixed reviewed implementation fence, not a sliding lookback and not
        # a dependency on permanent retention of an old run object. All eligible
        # reruns stay inside this interval; source ancestry excludes old code.
        created = INCEPTION_DATE + 'T00:00:00Z'
        runs = run_inventory(transport, created)
    except Exception:
        block(missing, 'prior-submission-history-incomplete')
        return result
    executions = 0
    family_cache = {source['sha']: (source['tree'], family)}
    relation_cache = {source['sha']: True, INCEPTION: True}
    def inventory(sha):
        if sha not in family_cache:
            transport.remaining()
            historical_source = read_source(sha)
            check(historical_source['sha'] == sha, 'History source snapshot differs')
            family_cache[sha] = (historical_source['tree'], selector.records(historical_source, 'npm'))
            del historical_source
            transport.remaining()
        tree, historical_family = family_cache[sha]
        overlap = {key for key in missing if key in historical_family and family[key]['version'] == historical_family[key]['version']}
        return {'sha': sha, 'tree': tree}, historical_family, overlap
    for listed in runs:
        overlap, sha = set(missing), None
        try:
            transport.remaining()
            check(isinstance(listed.get('event'), str) and isinstance(listed.get('head_branch'), str)
                  and integer(listed.get('repository', {}).get('id')) and integer(listed.get('head_repository', {}).get('id')), 'History run origin incomplete')
            if listed['event'] not in ['push', 'workflow_dispatch'] or listed['head_branch'] != repository['default_branch']:
                continue
            if listed['repository']['id'] != repository['id'] or listed['head_repository']['id'] != repository['id']:
                continue
            verify_run(listed, repository)
            sha = listed['head_sha']
            if sha not in relation_cache:
                relation_cache[sha] = inception_relation(transport, sha)
            if not relation_cache[sha]:
                continue
            check(listed['run_attempt'] <= MAX_ATTEMPTS, 'History attempt bound reached')
            for attempt in range(1, listed['run_attempt'] + 1):
                if current and listed['id'] == current['runId'] and attempt == current['attempt']:
                    continue
                executions += 1
                check(executions <= MAX_EXECUTIONS, 'History execution bound reached')
                try:
                    run = transport.get('/actions/runs/' + str(listed['id']) + '/attempts/' + str(attempt))
                    verify_run(run, repository, listed['id'], attempt)
                    check(run['head_sha'] == sha, 'History attempt source differs')
                    job = publisher_job(transport, run)
                    if job is None:
                        continue
                    # An authenticated skipped publisher needs no source inventory.
                    # Only potentially executed publishers can taint exact versions.
                    historical, historical_family, overlap = inventory(sha)
                    if not overlap:
                        continue
                    rows, artifact = report_for(transport, run, job, historical, historical_family)
                    possible = {key for key in overlap if possible_submission(rows[key])}
                    block(possible, 'prior-submission-unconfirmed', {'runId': run['id'], 'attempt': attempt,
                        'sourceSHA': sha, 'artifactId': artifact['id'], 'artifactDigest': artifact['digest']})
                except HistoryDeadline:
                    block(missing, 'prior-submission-history-incomplete')
                    return result
                except Exception:
                    try:
                        _, _, overlap = inventory(sha)
                    except HistoryDeadline:
                        block(missing, 'prior-submission-history-incomplete')
                        return result
                    except Exception:
                        pass
                    block(overlap, 'prior-submission-history-unavailable')
        except HistoryDeadline:
            block(missing, 'prior-submission-history-incomplete')
            return result
        except Exception:
            if sha is not None:
                try:
                    _, _, overlap = inventory(sha)
                except Exception:
                    pass
            block(overlap, 'prior-submission-history-incomplete')
    selector.validate(source, 'npm', result)
    return result


@contextmanager
def interrupt_deadline(seconds):
    """Interrupt blocking socket, archive and Git reads in the Linux planner."""
    check(isinstance(seconds, (int, float)) and not isinstance(seconds, bool) and 0 < seconds <= 180, 'Invalid history deadline')
    check(hasattr(signal, 'setitimer'), 'History deadline requires a supported planner runtime')
    previous_handler = signal.getsignal(signal.SIGALRM)
    previous_delay, previous_interval = signal.getitimer(signal.ITIMER_REAL)
    started = time.monotonic()
    def interrupt(_number, _frame):
        raise HistoryDeadline('History read deadline exceeded')
    # An already-shorter outer deadline retains its own handler and authority.
    own = not previous_delay or seconds < previous_delay
    if own:
        signal.signal(signal.SIGALRM, interrupt)
        signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    finally:
        if own:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, previous_handler)
            if previous_delay:
                signal.setitimer(signal.ITIMER_REAL, max(0.001, previous_delay - (time.monotonic() - started)), previous_interval)


def guard(source, snapshot, transport, *, current=None, read_source=evidence.source_reader, clock=time.monotonic, seconds=180):
    if not any(row.get('status') == 'missing' for row in snapshot.get('packages', {}).values()):
        # Current HTTP 200 packages do not depend on history or its availability.
        return _guard(source, snapshot, transport, current=current, read_source=read_source, clock=clock, seconds=seconds)
    try:
        with interrupt_deadline(seconds):
            return _guard(source, snapshot, transport, current=current, read_source=read_source, clock=clock, seconds=seconds)
    except HistoryDeadline:
        result = copy.deepcopy(snapshot)
        for row in result['packages'].values():
            if row.get('status') == 'missing':
                row.update(status='unknown', reason='prior-submission-history-incomplete')
        return result
