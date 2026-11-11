"""Authenticated historical submissions never become upload or recovery authority."""
# cspell:words importlib SIGALRM ITIMER getitimer ENEEDAUTH ETIMEDOUT
import copy
import hashlib
import importlib.util
import io
import json
import signal
import subprocess
import sys
import time
from pathlib import Path
import unittest
import urllib.parse
from unittest import mock
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


history = load('history_contract', 'release-version-history.py')
fixtures = load('history_fixtures', 'release-registry-selection.test.py')


def outcome(name, possible=True):
    value = {'name': name, 'version': '1.0.0', 'status': 'failed', 'uploadAttempted': possible,
             'outcomeUncertain': possible, 'registryBytesVerified': False, 'errors': []}
    return value


def rejection(name):
    return {**outcome(name), 'errors': [{'stage': 'publish', 'code': 'publish-failed',
        'message': 'npm operation=publish package=' + name + '@1.0.0 exit=1; ENEEDAUTH: Registry authentication required; outcome uncertain: inspect independent registry state before resuming'}]}


def archive_bytes(report, *, entries=None):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, body in entries or [('results.json', json.dumps(report))]:
            archive.writestr(name, body)
    return buffer.getvalue()


class Contracts(unittest.TestCase):
    def setUp(self):
        self.source, _ = fixtures.source_fixture()
        self.source['sha'] = history.INCEPTION
        self.repository = {'id': 5, 'default_branch': 'main'}
        self.run = {'id': 10, 'run_attempt': 1, 'repository': {'id': 5}, 'head_repository': {'id': 5},
                    'head_sha': history.INCEPTION, 'path': history.WORKFLOW, 'head_branch': 'main',
                    'event': 'push', 'status': 'completed', 'conclusion': 'failure', 'created_at': '2000-01-01T00:00:00Z'}
        self.job = {'id': 100, 'run_id': 10, 'run_attempt': 1, 'head_sha': history.INCEPTION,
                    'name': history.PUBLISHER, 'status': 'completed', 'conclusion': 'failure',
                    'steps': [{'name': history.PUBLISH_STEP, 'status': 'completed', 'conclusion': 'failure'},
                              {'name': history.UPLOAD_STEP, 'status': 'completed', 'conclusion': 'success',
                               'started_at': '2026-11-11T00:00:00Z', 'completed_at': '2026-11-11T00:01:00Z'}]}
        self.report = {'schemaVersion': 1, 'kind': 'npm-package-publication-results', 'stage': 'publisher',
            'sourceSHA': self.source['sha'], 'sourceTree': self.source['tree'], 'runId': 10, 'attempt': 1,
            'producerRunId': 10, 'producerAttempt': 1, 'manifestHash': 'd' * 64,
            'packages': [outcome('a'), rejection('b')]}
        self.snapshot = history.selector.collect(self.source, 'npm', fixtures.observed)
        self.listed = [self.run]
        self.artifact_changes = {}
        self.report_changes = {}
        self.job_changes = {}
        self.transport = mock.Mock()
        self.transport.get.side_effect = self.get
        self.transport.archive.side_effect = self.body

    def report_at(self, attempt):
        report = copy.deepcopy(self.report)
        report['attempt'] = attempt
        report.update(self.report_changes.get(attempt, {}))
        return report

    def body(self, artifact_id):
        return archive_bytes(self.report_at(artifact_id - 1000))

    def artifact_at(self, attempt):
        body = self.body(1000 + attempt)
        return {'id': 1000 + attempt, 'name': 'npm-publication-results-10-' + str(attempt),
                'expired': False, 'workflow_run': {'id': 10, 'head_sha': self.source['sha'], 'repository_id': 5, 'head_repository_id': 5},
                'size_in_bytes': len(body), 'digest': 'sha256:' + hashlib.sha256(body).hexdigest(),
                'created_at': '2026-11-11T00:00:30Z', **self.artifact_changes}

    def get(self, route):
        if route == '':
            return self.repository
        if '/workflows/' in route:
            return {'total_count': len(self.listed), 'workflow_runs': self.listed}
        if '/attempts/' in route:
            attempt = int(route.split('/attempts/')[1].split('/')[0])
            if '/jobs?' in route:
                job = copy.deepcopy(self.job)
                job['run_attempt'] = attempt
                job.update(self.job_changes.get(attempt, {}))
                return {'jobs': [job]}
            return {**self.run, 'run_attempt': attempt}
        if '/runs/10/artifacts?' in route:
            attempt = int(route.rsplit('-', 1)[1])
            return {'total_count': 1, 'artifacts': [self.artifact_at(attempt)]}
        if '/actions/artifacts/' in route:
            return self.artifact_at(int(route.rsplit('/', 1)[1]) - 1000)
        raise AssertionError(route)

    def guard(self, **options):
        return history.guard(self.source, self.snapshot, self.transport, read_source=lambda sha: self.source, **options)

    def test_ambiguous_blocks_only_exact_version_and_legacy_auth_rejection_does_not(self):
        result = self.guard()
        self.assertEqual(result['packages']['npm:a']['reason'], 'prior-submission-unconfirmed')
        self.assertEqual(result['packages']['npm:b']['status'], 'missing')
        self.assertEqual(result['packages']['npm:a']['priorSubmission']['artifactId'], 1001)
        self.assertEqual(self.snapshot['packages']['npm:a']['status'], 'missing')
        self.assertNotIn('baseline', json.dumps(result))

    def test_old_run_rerun_is_enumerated_and_later_rejection_does_not_clear(self):
        self.run['run_attempt'] = 2
        self.report_changes[2] = {'packages': [rejection('a'), rejection('b')]}
        result = self.guard()
        self.assertEqual(result['packages']['npm:a']['reason'], 'prior-submission-unconfirmed')
        self.assertEqual(result['packages']['npm:a']['priorSubmission']['attempt'], 1)
        self.assertTrue(any('/attempts/2/jobs?' in call.args[0] for call in self.transport.get.call_args_list))

    def test_current_attempt_is_excluded_but_original_attempt_is_not(self):
        self.run['run_attempt'] = 2
        result = self.guard(current={'runId': 10, 'attempt': 2})
        self.assertEqual(result['packages']['npm:a']['reason'], 'prior-submission-unconfirmed')
        self.assertFalse(any('/attempts/2/' in call.args[0] for call in self.transport.get.call_args_list))

    def test_current_200_never_inherits_history_failure(self):
        self.snapshot['packages']['npm:a'] = fixtures.observed(history.selector.records(self.source, 'npm')['npm:a'], True)
        self.artifact_changes['expired'] = True
        result = self.guard()
        self.assertEqual(result['packages']['npm:a']['status'], 'published')
        self.assertEqual(result['packages']['npm:b']['reason'], 'prior-submission-history-unavailable')
        self.snapshot['packages']['npm:b'] = fixtures.observed(history.selector.records(self.source, 'npm')['npm:b'], True)
        self.transport.reset_mock()
        self.assertEqual(self.guard(), self.snapshot)
        self.transport.get.assert_not_called()

    def test_no_publish_step_execution_needs_no_report(self):
        self.job['steps'][0]['conclusion'] = 'skipped'
        self.artifact_changes['expired'] = True
        self.assertEqual(self.guard(), self.snapshot)
        self.transport.archive.assert_not_called()

    def test_failed_run_does_not_prove_no_submission(self):
        self.run['conclusion'] = 'cancelled'
        self.job['conclusion'] = 'cancelled'
        self.assertEqual(self.guard()['packages']['npm:a']['status'], 'unknown')

    def test_missing_corrupt_expired_or_wrong_attempt_report_fails_closed(self):
        for patch in [{'expired': True}, {'digest': 'sha256:' + 'f' * 64}, {'workflow_run': {'id': 11}},
                      {'name': 'npm-publication-results-10-2'}]:
            with self.subTest(patch=patch):
                self.artifact_changes = patch
                result = self.guard()
                self.assertEqual({row['reason'] for row in result['packages'].values()}, {'prior-submission-history-unavailable'})
        self.artifact_changes = {}
        for patch in [{'attempt': 2}, {'sourceSHA': 'f' * 40}, {'packages': [outcome('a'), outcome('a')]},
                      {'packages': [outcome('a')]}]:
            with self.subTest(patch=patch):
                self.report_changes[1] = patch
                self.assertEqual({row['status'] for row in self.guard()['packages'].values()}, {'unknown'})

    def test_wrong_job_identity_or_upload_binding_cannot_supply_safe_report(self):
        self.report['packages'] = [rejection('a'), rejection('b')]
        for patch in [{'run_id': 11}, {'run_attempt': 2}, {'head_sha': 'f' * 40}]:
            self.job_changes[1] = patch
            self.assertEqual({row['status'] for row in self.guard()['packages'].values()}, {'unknown'})
        self.job_changes = {}
        self.job['steps'][1]['conclusion'] = 'failure'
        self.assertEqual({row['status'] for row in self.guard()['packages'].values()}, {'unknown'})

    def test_untrusted_other_branch_fork_and_pr_are_not_publication_authority(self):
        for field, value in [('event', 'pull_request'), ('head_branch', 'feature'), ('head_repository', {'id': 6})]:
            run = copy.deepcopy(self.run)
            run[field] = value
            self.listed = [run]
            self.assertEqual(self.guard(), self.snapshot)

    def test_paging_incomplete_or_duplicate_inventory_is_explicit_unknown(self):
        for listing in [{'total_count': 2, 'workflow_runs': [self.run]}, {'total_count': 2, 'workflow_runs': [self.run, self.run]}]:
            old = self.get
            self.transport.get.side_effect = lambda route: listing if '/workflows/' in route else old(route)
            self.assertEqual({row['reason'] for row in self.guard()['packages'].values()}, {'prior-submission-history-incomplete'})
        listing = {'total_count': 101, 'workflow_runs': [{**self.run, 'id': index + 1} for index in range(100)]}
        self.transport.get.side_effect = lambda route: listing if '/workflows/' in route else self.get(route)
        with mock.patch.object(history, 'MAX_PAGES', 1):
            self.assertEqual({row['reason'] for row in self.guard()['packages'].values()}, {'prior-submission-history-incomplete'})

    def test_report_failure_is_limited_to_overlapping_exact_family(self):
        historical = copy.deepcopy(self.source)
        current = copy.deepcopy(self.source)
        package = json.loads(current['files']['packages/b/package.json'])
        package['version'] = '2.0.0'
        current['files']['packages/b/package.json'] = json.dumps(package).encode()
        family = json.loads(current['files']['docs/releases/v2-candidate.json'])
        next(row for row in family['npm']['primary'] if row['name'] == 'b')['version'] = '2.0.0'
        current['files']['docs/releases/v2-candidate.json'] = json.dumps(family).encode()
        current['sha'] = 'e' * 40
        self.snapshot = history.selector.collect(current, 'npm', fixtures.observed)
        self.artifact_changes['expired'] = True
        old_get = self.get
        self.transport.get.side_effect = lambda route: {'base_commit': {'sha': history.INCEPTION}, 'merge_base_commit': {'sha': history.INCEPTION}, 'status': 'ahead'} if route.startswith('/compare/') else old_get(route)
        result = history.guard(current, self.snapshot, self.transport, read_source=lambda sha: historical)
        self.assertEqual(result['packages']['npm:a']['status'], 'unknown')
        self.assertEqual(result['packages']['npm:b']['status'], 'missing')

    def test_prior_source_before_inception_excludes_rerun_by_code_not_date(self):
        self.run['head_sha'] = 'e' * 40
        self.run['run_attempt'] = 20
        old_get = self.get
        self.transport.get.side_effect = lambda route: {'base_commit': {'sha': history.INCEPTION}, 'merge_base_commit': {'sha': 'e' * 40}, 'status': 'behind'} if route.startswith('/compare/') else old_get(route)
        self.assertEqual(self.guard(), self.snapshot)
        self.transport.archive.assert_not_called()

    def test_legacy_rejection_is_exact_and_cannot_hide_timeouts_or_other_operations(self):
        self.assertFalse(history.possible_submission(rejection('a')))
        for text in ['timeout ' + rejection('a')['errors'][0]['message'],
                     rejection('a')['errors'][0]['message'].replace('operation=publish', 'operation=dist-tag'),
                     rejection('a')['errors'][0]['message'].replace('package=a@', 'package=b@'),
                     rejection('a')['errors'][0]['message'] + '\nextra ENEEDAUTH']:
            row = rejection('a')
            row['errors'][0]['message'] = text
            self.assertTrue(history.possible_submission(row))
        row = rejection('a')
        row['errors'].append({'stage': 'publish', 'code': 'timeout'})
        self.assertTrue(history.possible_submission(row))
        row = rejection('a')
        row['accepted'] = True
        self.assertTrue(history.possible_submission(row))
        row = rejection('a')
        row.update(uploadAttempted=False, outcomeUncertain=False)
        self.assertTrue(history.possible_submission(row))

    def test_structured_rejection_requires_publish_non_timeout_and_consistent_flags(self):
        row = {**outcome('a'), 'accepted': False, 'outcomeUncertain': False,
            'publishOutcome': {'operation': 'publish', 'returncode': 1, 'timedOut': False,
                               'npmCodes': ['ENEEDAUTH'], 'definiteRejection': True, 'accepted': False}}
        self.assertFalse(history.possible_submission(row))
        for field, value in [('operation', 'dist-tag'), ('timedOut', True), ('returncode', 0), ('returncode', -9),
                             ('npmCodes', ['ENEEDAUTH', 'ETIMEDOUT']), ('definiteRejection', False)]:
            changed = copy.deepcopy(row)
            changed['publishOutcome'][field] = value
            self.assertTrue(history.possible_submission(changed))
        for field, value in [('outcomeUncertain', True), ('uploadAttempted', False), ('accepted', True)]:
            changed = copy.deepcopy(row)
            changed[field] = value
            self.assertTrue(history.possible_submission(changed))

    def test_success_or_accepted_contradictions_never_mean_safe_rejection(self):
        for base in [rejection('a'), outcome('a', False), {**outcome('a'), 'outcomeUncertain': False,
            'publishOutcome': {'operation': 'publish', 'returncode': 1, 'timedOut': False,
                'npmCodes': ['ENEEDAUTH'], 'definiteRejection': True, 'accepted': False}}]:
            for field, value in [('publishCommandSucceeded', True), ('registryBytesVerified', True), ('status', 'success')]:
                row = copy.deepcopy(base)
                row[field] = value
                self.assertTrue(history.possible_submission(row))

    def test_fixed_inception_date_excludes_pre_feature_page_cap_without_expiring_history(self):
        self.guard()
        routes = [call.args[0] for call in self.transport.get.call_args_list if '/workflows/' in call.args[0]]
        self.assertEqual(len(routes), 1)
        self.assertIn('branch=main&created=%3E%3D2026-11-11', routes[0])
        self.assertNotIn('status=completed', routes[0])
        self.assertEqual(self.guard()['packages']['npm:a']['status'], 'unknown')

    def test_history_deadline_returns_unknown_instead_of_aborting_plan(self):
        times = iter([0, 0, 181])
        result = self.guard(clock=lambda: next(times, 181))
        self.assertEqual({row['reason'] for row in result['packages'].values()}, {'prior-submission-history-incomplete'})
        self.assertLess(len(self.transport.get.call_args_list), 3)

    def test_skipped_publisher_does_not_read_invalid_historical_source(self):
        self.run['head_sha'] = 'e' * 40
        self.job['head_sha'] = 'e' * 40
        self.job['steps'][0]['conclusion'] = 'skipped'
        old_get = self.get
        self.transport.get.side_effect = lambda route: {'base_commit': {'sha': history.INCEPTION}, 'merge_base_commit': {'sha': history.INCEPTION}, 'status': 'ahead'} if route.startswith('/compare/') else old_get(route)
        read_source = mock.Mock(side_effect=ValueError('source was not publication ready'))
        self.assertEqual(history.guard(self.source, self.snapshot, self.transport, read_source=read_source), self.snapshot)
        read_source.assert_not_called()

    def test_date_partitioning_completes_more_than_one_api_capacity(self):
        runs = [{**self.run, 'id': index + 1, 'created_at': '2026-11-11T00:00:00Z' if index < 75 else '2026-11-11T23:59:59Z'} for index in range(150)]
        runs.sort(key=lambda row: row['created_at'], reverse=True)
        def get(route):
            query = urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)
            window = query['created'][0]
            selected = runs if window.startswith('>=') else [row for row in runs if window.split('..')[0] <= row['created_at'] <= window.split('..')[1]]
            page = int(query['page'][0])
            return {'total_count': len(selected), 'workflow_runs': selected[(page - 1) * 100:page * 100]}
        transport = mock.Mock()
        transport.get.side_effect = get
        with mock.patch.object(history, 'MAX_PAGES', 1):
            result = history.run_inventory(transport, '2026-11-11T05:15:20Z')
        self.assertEqual({row['id'] for row in result}, set(range(1, 151)))
        self.assertTrue(any('..' in urllib.parse.unquote(call.args[0]) for call in transport.get.call_args_list))

    def test_actual_presence_report_schema_and_nested_publish_outcome(self):
        actual = load('actual_presence_history_fixture', 'release-npm-version-presence.test.py')
        manifest, bodies, registry = actual.fixture(packages=[('a', {}), ('b', {})])
        registry.fail.add('b')
        report, outcomes = actual.execute(manifest, bodies, registry)
        self.assertEqual(report['schemaVersion'], 2)
        self.assertTrue(history.possible_submission(outcomes['a']))
        self.assertFalse(history.possible_submission(outcomes['b']))
        self.assertIs(outcomes['b']['publishOutcome']['accepted'], False)
        for name in ['a', 'b']:
            file = 'packages/' + name + '/package.json'
            package = json.loads(self.source['files'][file])
            package['version'] = '2.0.0'
            self.source['files'][file] = json.dumps(package).encode()
        family = json.loads(self.source['files']['docs/releases/v2-candidate.json'])
        for row in family['npm']['primary']:
            row['version'] = '2.0.0'
        self.source['files']['docs/releases/v2-candidate.json'] = json.dumps(family).encode()
        self.snapshot = history.selector.collect(self.source, 'npm', fixtures.observed)
        self.report = {**report, 'runId': 10, 'attempt': 1, 'producerRunId': 10, 'producerAttempt': 1,
                       'sourceSHA': self.source['sha'], 'sourceTree': self.source['tree']}
        result = self.guard()
        self.assertEqual(result['packages']['npm:a']['reason'], 'prior-submission-unconfirmed')
        self.assertEqual(result['packages']['npm:b']['status'], 'missing')

    def test_real_deadline_interrupts_blocking_transport_and_restores_signal(self):
        previous = signal.getsignal(signal.SIGALRM)
        calls = []
        def blocked(_route):
            calls.append(True)
            time.sleep(2)
            raise AssertionError('blocking call was not interrupted')
        self.transport.get.side_effect = blocked
        start = time.monotonic()
        result = self.guard(seconds=0.05)
        self.assertLess(time.monotonic() - start, 0.5)
        self.assertEqual({row['reason'] for row in result['packages'].values()}, {'prior-submission-history-incomplete'})
        self.assertEqual(signal.getsignal(signal.SIGALRM), previous)
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL)[0], 0)
        self.assertEqual(len(calls), 1)

    def test_real_deadline_interrupts_blocking_source_reader(self):
        self.run['head_sha'] = 'e' * 40
        self.job['head_sha'] = 'e' * 40
        old_get = self.get
        self.transport.get.side_effect = lambda route: {'base_commit': {'sha': history.INCEPTION}, 'merge_base_commit': {'sha': history.INCEPTION}, 'status': 'ahead'} if route.startswith('/compare/') else old_get(route)
        read_source = mock.Mock(side_effect=lambda sha: time.sleep(2))
        start = time.monotonic()
        result = history.guard(self.source, self.snapshot, self.transport, read_source=read_source, seconds=0.05)
        self.assertLess(time.monotonic() - start, 0.5)
        self.assertEqual({row['status'] for row in result['packages'].values()}, {'unknown'})
        read_source.assert_called_once()

    def test_deadline_in_real_subprocess_does_not_wait_for_blocking_read(self):
        script = ("import importlib.util, time\n"
                  + "s=importlib.util.spec_from_file_location('history', " + repr(str(Path(__file__).with_name('release-version-history.py'))) + ")\n"
                  + "m=importlib.util.module_from_spec(s); s.loader.exec_module(m)\n"
                  + "try:\n with m.interrupt_deadline(0.05): time.sleep(5)\n"
                  + "except m.HistoryDeadline: print('interrupted')\n")
        result = subprocess.run([sys.executable, '-B', '-c', script], capture_output=True, text=True, timeout=2, check=True)
        self.assertEqual(result.stdout.strip(), 'interrupted')

    def test_deleted_inception_run_is_not_a_permanent_planning_gate(self):
        result = self.guard()
        self.assertEqual(result['packages']['npm:b']['status'], 'missing')
        self.assertEqual(result['packages']['npm:a']['reason'], 'prior-submission-unconfirmed')
        self.assertFalse(any(call.args[0] == '/actions/runs/42964546726' for call in self.transport.get.call_args_list))
        self.assertEqual(history.INCEPTION_DATE, '2026-11-11')

    def test_zip_member_bounds_paths_duplicates_and_json_duplicate_keys_reject(self):
        for entries in [[('../results.json', '{}')], [('results.json', '{}'), ('extra.json', '{}')],
                        [('results.json', '{"packages":[],"packages":[]}')]]:
            body = archive_bytes({}, entries=entries)
            artifact = {'size_in_bytes': len(body), 'digest': 'sha256:' + hashlib.sha256(body).hexdigest()}
            with self.assertRaises(ValueError):
                history.decode_report(body, artifact)


if __name__ == '__main__':
    unittest.main()
