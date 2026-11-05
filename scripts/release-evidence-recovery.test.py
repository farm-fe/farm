"""Actual wait -> pending bytes -> recovery -> original retry, with no network."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('recovery_tests', Path(__file__).with_name('release-evidence-recovery.py'))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
SHA = 'a' * 40
TREE = 'b' * 40


def run(number, path, conclusion='success', attempt=1):
    return {'workflow_id': {m.WORKFLOW: 1, '.github/workflows/ci.yaml': 2, '.github/workflows/rust-test.yaml': 3}[path], 'id': number, 'path': path, 'head_sha': SHA, 'head_branch': 'main', 'event': 'push',
            'run_attempt': attempt, 'status': 'completed', 'conclusion': conclusion,
            'repository': {'id': m.REPOSITORY_ID, 'full_name': m.REPOSITORY}, 'head_repository': {'id': m.REPOSITORY_ID}}


class Clock:
    def __init__(self): self.value = 0
    def monotonic(self): return self.value
    def sleep(self, seconds): self.value += seconds


class LocalActions:
    def __init__(self, receipt=None):
        self.release = run(50, m.WORKFLOW, 'failure')
        self.ci = run(10, '.github/workflows/ci.yaml', attempt=3)
        self.unit = run(20, '.github/workflows/rust-test.yaml', attempt=2)
        self.rows = [self.release]; self.posts = []; self.ready = True; self.clock = None; self.ready_at = 0
        self.job = {'id': 100, 'run_id': 50, 'run_attempt': 1, 'head_sha': SHA, 'name': m.EVIDENCE_JOB,
                    'status': 'completed', 'conclusion': 'failure', 'steps': [{'name': m.UPLOAD,
                    'status': 'completed', 'conclusion': 'success', 'started_at': '2026-11-05T02:00:00Z', 'completed_at': '2026-11-05T02:00:02Z'}]}
        self.jobs = [self.job, *[{'id': number, 'run_id': 50, 'run_attempt': 1, 'head_sha': SHA, 'name': name,
                    'status': 'completed', 'conclusion': 'skipped', 'steps': []} for number, name in [
                    (101, m.functional.entry.PRODUCER), (102, 'Publish exact independent release archives under durable lock')]]]
        record = receipt or m.functional.pending_record({'sha': SHA, 'tree': TREE}, {'runId': 50, 'attempt': 1})
        body = io.BytesIO()
        with zipfile.ZipFile(body, 'w') as archive: archive.writestr('release-functional-pending.json', json.dumps(record))
        self.body = body.getvalue()
        self.artifact = {'id': 900, 'name': 'release-functional-pending-50-1', 'expired': False,
                         'workflow_run': {'id': 50, 'head_sha': SHA}, 'created_at': '2026-11-05T02:00:01Z',
                         'size_in_bytes': len(self.body), 'digest': 'sha256:' + hashlib.sha256(self.body).hexdigest()}
        self.artifacts = [self.artifact]

    def get(self, route):
        if route == '': return {'id': m.REPOSITORY_ID, 'full_name': m.REPOSITORY, 'default_branch': 'main'}
        if route.startswith('/actions/workflows/') and '/runs?' not in route:
            name = route.split('/')[-1]; return {'id': {'incremental-release.yaml':1,'ci.yaml':2,'rust-test.yaml':3}[name], 'path': '.github/workflows/'+name}
        if route.startswith('/actions/workflows/'):
            name = route.split('/')[3]; rows = {'incremental-release.yaml': self.rows, 'ci.yaml': [self.ci], 'rust-test.yaml': [self.unit]}[name]
            rows = copy.deepcopy(rows)
            if name != 'incremental-release.yaml' and (not self.ready or self.clock and self.clock.value < self.ready_at):
                rows[0].update(status='in_progress', conclusion=None)
            return {'total_count': len(rows), 'workflow_runs': rows}
        if route == '/actions/runs/50': return copy.deepcopy(self.release)
        if route == '/actions/runs/10': return copy.deepcopy(self.ci)
        if route == '/actions/runs/20': return copy.deepcopy(self.unit)
        if route in ['/actions/runs/50/attempts/1','/actions/runs/50/attempts/2']: return copy.deepcopy(self.release)
        if route.endswith('/jobs?per_page=100'): return {'total_count': len(self.jobs), 'jobs': copy.deepcopy(self.jobs)}
        if route.startswith('/actions/runs/50/artifacts?'):
            name = route.split('&name=')[1]; rows = [a for a in self.artifacts if a['name'] == name]
            return {'total_count': len(rows), 'artifacts': copy.deepcopy(rows)}
        if route == '/actions/artifacts/900': return copy.deepcopy(self.artifact)
        if route == '/git/commits/' + SHA: return {'tree': {'sha': TREE}}
        raise AssertionError(route)

    def archive(self, number):
        assert number == 900
        return self.body

    def rerun(self, number):
        assert number == 50
        self.posts.append(number)
        self.release.update(status='queued', conclusion=None, run_attempt=2)

    def event(self, scope='release'):
        return {'action': 'completed', 'workflow_run': copy.deepcopy(getattr(self, scope))}


class Recovery(unittest.TestCase):
    def test_deadline_boundaries_and_later_success_use_actual_wait_and_recovery(self):
        for ready in [7199, 7200, 7201]:
            with self.subTest(ready=ready):
                clock = Clock(); transport = LocalActions(); transport.clock = clock; transport.ready_at = ready
                source = {'sha': SHA, 'tree': TREE}; current = {'runId': 50, 'attempt': 1}
                if ready <= 7200:
                    result = m.functional.wait_for_main(transport, source, current, seconds=7200, clock=clock.monotonic, sleep=clock.sleep)
                    self.assertEqual(result['ci']['attempt'], 3); self.assertEqual(clock.value, 7200)
                else:
                    with self.assertRaisesRegex(ValueError, 'not ready'):
                        m.functional.wait_for_main(transport, source, current, seconds=7200, clock=clock.monotonic, sleep=clock.sleep)
                    self.assertEqual(clock.value, 7200)
                    pending = m.functional.pending_record(source, current)
                    # Transport uses the actual pending bytes, digest, original upload step and run.
                    transport = LocalActions(pending); event = transport.event()
                    result = m.recover(transport, event)
                    self.assertTrue(result['requested']); self.assertEqual(transport.posts, [50])
                    self.assertEqual(result['sourceSHA'], SHA)
                    self.assertEqual(m.recover(transport, event)['action'], 'none')
                    self.assertEqual(transport.posts, [50])

    def test_pending_then_ci_completion_and_release_completion_race(self):
        transport = LocalActions(); transport.ready = False
        self.assertEqual(m.recover(transport, transport.event())['action'], 'none')
        transport.ready = True
        self.assertTrue(m.recover(transport, transport.event('ci'))['requested'])
        self.assertEqual(transport.posts, [50])
        # If CI finishes while release is still running, release completion wakes recovery.
        transport = LocalActions(); transport.release['status'] = 'in_progress'
        self.assertEqual(m.recover(transport, transport.event('ci'))['action'], 'none')
        transport.release['status'] = 'completed'
        self.assertTrue(m.recover(transport, transport.event())['requested'])

    def test_failure_cancellation_and_new_attempt_never_accept_old_success(self):
        for status in ['failure', 'cancelled']:
            transport = LocalActions(); transport.ci['conclusion'] = status
            self.assertEqual(m.recover(transport, transport.event())['action'], 'none'); self.assertFalse(transport.posts)
            with self.assertRaisesRegex(ValueError, 'not ready'):
                m.functional.discover_main(transport, {'sha': SHA}, {'runId': 50, 'attempt': 1})
        transport = LocalActions(); transport.release['conclusion'] = 'cancelled'
        self.assertEqual(m.recover(transport, transport.event('ci'))['action'], 'none')
        transport = LocalActions(); event = transport.event('ci'); transport.ci.update(run_attempt=4, status='in_progress', conclusion=None)
        self.assertEqual(m.recover(transport, event)['action'], 'none')

    def test_source_repository_event_and_tree_tampering_reject(self):
        for mutate in [lambda t:t.release.update(head_sha='c'*40), lambda t:t.release.update(event='pull_request'), lambda t:t.ci.update(workflow_id=99),
                       lambda t:t.ci['head_repository'].update(id=9), lambda t:t.artifact['workflow_run'].update(head_sha='c'*40)]:
            transport = LocalActions(); event = transport.event(); mutate(transport)
            with self.assertRaises(ValueError): m.recover(transport, event)
            self.assertFalse(transport.posts)
        receipt = m.functional.pending_record({'sha': SHA, 'tree': 'c'*40}, {'runId': 50, 'attempt': 1})
        with self.assertRaisesRegex(ValueError, 'receipt source'): m.recover(LocalActions(receipt), LocalActions().event())

    def test_missing_corrupt_expired_or_unbound_pending_receipt_never_reruns(self):
        transport = LocalActions(); transport.artifacts = []
        self.assertEqual(m.recover(transport, transport.event())['action'], 'none')
        for mutate in [lambda t:setattr(t,'body',t.body+b'corrupt'),lambda t:t.artifact.update(expired=True),
                       lambda t:t.artifact.update(created_at='2026-11-05T02:00:03Z'),
                       lambda t:t.job.update(head_sha='c'*40),lambda t:t.job['steps'][0].update(conclusion='failure')]:
            transport = LocalActions(); mutate(transport)
            with self.assertRaises(ValueError): m.recover(transport, transport.event())
            self.assertFalse(transport.posts)

    def test_frozen_transaction_and_possible_publisher_never_rebuild(self):
        for name in ['farm-release-candidates-'+SHA+'-1', 'farm-release-seal-'+SHA+'-1']:
            transport = LocalActions(); transport.artifacts.append({**transport.artifact, 'name':name, 'id':901})
            result = m.recover(transport, transport.event()); self.assertEqual(result['reason'], 'frozen-transaction-use-original-recovery')
            self.assertFalse(transport.posts)
        for index in [1,2]:
            transport = LocalActions(); transport.jobs[index].update(conclusion='success')
            with self.assertRaises(ValueError): m.recover(transport, transport.event())
            self.assertFalse(transport.posts)

    def test_main_moving_does_not_change_original_transaction_source(self):
        transport = LocalActions()
        result = m.recover(transport, transport.event())
        self.assertEqual(result['sourceSHA'], SHA); self.assertEqual(transport.posts, [50])
        # No branch resolution/dispatch occurs: original run 50 determines checkout.
        self.assertNotIn('ref', result)

    def test_different_latest_run_and_recheck_race_fail_closed(self):
        transport = LocalActions(); transport.rows.append(run(51,m.WORKFLOW,'success'))
        self.assertEqual(m.recover(transport, transport.event('ci'))['action'], 'none')
        transport = LocalActions(); original = transport.get; count = 0
        def get(route):
            nonlocal count
            if route == '/actions/runs/50':
                count += 1
                if count == 2: transport.release.update(status='queued', conclusion=None, run_attempt=2)
            return original(route)
        transport.get = get
        self.assertEqual(m.recover(transport, transport.event())['action'], 'none'); self.assertFalse(transport.posts)

    def test_native_failure_or_cancellation_is_not_an_automatic_retry_channel(self):
        for state in ['failure','cancelled','timed_out']:
            transport = LocalActions()
            transport.jobs.append({'name':'native producer','status':'completed','conclusion':state})
            self.assertEqual(m.recover(transport,transport.event())['reason'],'another-job-requires-independent-repair')
            self.assertFalse(transport.posts)

    def test_native_artifact_volume_does_not_hide_frozen_candidates(self):
        transport = LocalActions()
        transport.artifacts.extend({'id':1000+i,'name':'native-'+str(i)} for i in range(400))
        self.assertTrue(m.recover(transport, transport.event())['requested'])
        transport = LocalActions()
        transport.artifacts.extend({'id':1000+i,'name':'native-'+str(i)} for i in range(400))
        transport.artifacts.append({'id':2000,'name':'farm-release-candidates-'+SHA+'-1'})
        self.assertEqual(m.recover(transport, transport.event())['reason'],'frozen-transaction-use-original-recovery')

    def test_retry_failure_without_current_pending_receipt_does_not_loop(self):
        transport = LocalActions()
        transport.release['run_attempt'] = 2
        transport.job['run_attempt'] = 2
        # A genuine verifier error has no pending upload in the new attempt.
        transport.job['steps'] = []
        self.assertEqual(m.recover(transport, transport.event())['reason'],'no-pending-evidence-receipt')
        self.assertFalse(transport.posts)

    def test_unfrozen_retry_rebuilds_only_original_fixed_source(self):
        transport = LocalActions()
        proof = m.functional.retry.verify_unfrozen_retry(transport.get, run_id=50, source_sha=SHA, current_attempt=2,
            workflow=m.WORKFLOW, producer=m.functional.entry.PRODUCER, freeze_step='Freeze independent release candidates',
            publisher='Publish exact independent release archives under durable lock', events=['push'])
        self.assertEqual(proof, [{'runId':50,'attempt':1,'publisher':'skipped','freeze':'not-executed'}])


if __name__ == '__main__': unittest.main(verbosity=2)
