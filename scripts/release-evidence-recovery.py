"""Resume only an unfrozen, fixed-main release stalled on functional evidence.

The completed workflow event is a wake-up, not publication authorization.
The only write is rerun for the independently verified original run.
"""
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


functional = load('recovery_functional', 'incremental-release-functional.py')
check = functional.check
WORKFLOW = '.github/workflows/incremental-release.yaml'
RECOVERY = '.github/workflows/release-evidence-recovery.yaml'
EVIDENCE_JOB = 'Verify actual fixed-main functional evidence'
UPLOAD = 'Preserve pending fixed-main evidence for bounded recovery'
ALLOWED = {WORKFLOW, '.github/workflows/ci.yaml', '.github/workflows/rust-test.yaml'}
REPOSITORY = 'farm-fe/farm'
REPOSITORY_ID = 507542208


def trusted_run(run, path=None, sha=None):
    check(run.get('path') in ALLOWED and (path is None or run['path'] == path), 'Unexpected wake-up workflow')
    check(run.get('event') == 'push' and run.get('head_branch') == 'main'
          and run.get('repository', {}).get('id') == REPOSITORY_ID == run.get('head_repository', {}).get('id')
          and run['repository'].get('full_name') == REPOSITORY, 'Not an original same-repository main push')
    check(re.fullmatch('[a-f0-9]{40}', run.get('head_sha', '')) and (sha is None or run['head_sha'] == sha), 'Recovery source differs')
    check(type(run.get('id')) is int and run['id'] > 0 and type(run.get('run_attempt')) is int and 1 <= run['run_attempt'] <= 20, 'Invalid recovery run identity')


def latest(get, workflow, sha):
    response = get('/actions/workflows/' + workflow + '/runs?per_page=20&event=push&head_sha=' + sha)
    rows = response['workflow_runs']
    check(type(response.get('total_count')) is int and response['total_count'] == len(rows) and len(rows) <= 20, 'Recovery run listing incomplete')
    definition = get('/actions/workflows/' + workflow)
    check(definition.get('path') == '.github/workflows/' + workflow and type(definition.get('id')) is int and definition['id'] > 0, 'Workflow definition differs')
    for run in rows:
        trusted_run(run, '.github/workflows/' + workflow, sha)
        check(run.get('workflow_id') == definition['id'], 'Run workflow ID differs')
    return max(rows, key=lambda row: row['id']) if rows else None


def decision(transport, event):
    check(event.get('action') == 'completed', 'Only completed workflow wake-ups are allowed')
    event_run = event['workflow_run']; trusted_run(event_run)
    wake = transport.get('/actions/runs/' + str(event_run['id']))
    trusted_run(wake, event_run['path'], event_run['head_sha'])
    definition = transport.get('/actions/workflows/' + wake['path'].split('/')[-1])
    check(definition.get('path') == wake['path'] and type(definition.get('id')) is int and definition['id'] > 0
          and wake.get('workflow_id') == event_run.get('workflow_id') == definition['id'], 'Wake-up workflow ID differs')
    # A delayed event for a previous attempt cannot act while a newer attempt runs.
    if wake['run_attempt'] != event_run['run_attempt'] or wake.get('status') != 'completed':
        return {'action': 'none', 'reason': 'wake-up-attempt-superseded'}
    sha = wake['head_sha']
    release = latest(transport.get, 'incremental-release.yaml', sha)
    if release is None or release.get('status') != 'completed' or release.get('conclusion') != 'failure':
        return {'action': 'none', 'reason': 'no-failed-release'}
    check(release['run_attempt'] < 20, 'Automatic recovery attempt bound exceeded')
    accepted = []
    for file in ['ci.yaml', 'rust-test.yaml']:
        run = latest(transport.get, file, sha)
        if run is None or run.get('status') != 'completed' or run.get('conclusion') != 'success':
            return {'action': 'none', 'reason': 'latest-functional-run-not-successful'}
        accepted.append({'id': run['id'], 'attempt': run['run_attempt'], 'path': run['path']})
    run_id, attempt = release['id'], release['run_attempt']
    route = '/actions/runs/' + str(run_id) + '/attempts/' + str(attempt)
    original = transport.get(route); trusted_run(original, WORKFLOW, sha)
    check(original.get('workflow_id') == release['workflow_id'] and original['id'] == run_id and original['run_attempt'] == attempt and original.get('status') == 'completed' and original.get('conclusion') == 'failure', 'Release attempt changed')
    response = transport.get(route + '/jobs?per_page=100'); jobs = response['jobs']
    check(response.get('total_count') == len(jobs) and len(jobs) <= 100, 'Recovery job listing incomplete')
    if any(job.get('status') != 'completed' or job.get('conclusion') not in ['success', 'skipped'] for job in jobs if job['name'] != EVIDENCE_JOB):
        return {'action': 'none', 'reason': 'another-job-requires-independent-repair'}
    found = [job for job in jobs if job['name'] == EVIDENCE_JOB]
    check(len(found) == 1, 'Evidence job missing or ambiguous')
    job = found[0]
    if job.get('conclusion') != 'failure': return {'action': 'none', 'reason': 'not-an-evidence-wait-failure'}
    check(job.get('run_id') == run_id and job.get('run_attempt') == attempt and job.get('head_sha') == sha and job.get('status') == 'completed', 'Evidence producer source differs')
    def named(name):
        response = transport.get('/actions/runs/' + str(run_id) + '/artifacts?per_page=2&name=' + name)
        check(response.get('total_count') == len(response['artifacts']) and len(response['artifacts']) <= 1
              and all(row['name'] == name for row in response['artifacts']), 'Exact recovery artifact lookup is ambiguous')
        return response['artifacts']
    # Native uploads exceed 100 artifacts. Query exact transaction names instead
    # of assuming the first page proves no frozen transaction exists.
    for prior in range(1, attempt + 1):
        for prefix in ['farm-release-candidates-', 'farm-release-seal-']:
            if named(prefix + sha + '-' + str(prior)):
                return {'action': 'none', 'reason': 'frozen-transaction-use-original-recovery'}
    name = 'release-functional-pending-' + str(run_id) + '-' + str(attempt)
    pending = named(name)
    if not pending: return {'action': 'none', 'reason': 'no-pending-evidence-receipt'}
    check(len(pending) == 1, 'Ambiguous pending receipt')
    artifact = transport.get('/actions/artifacts/' + str(pending[0]['id']))
    check(artifact['name'] == name and artifact.get('expired') is False and artifact['workflow_run']['id'] == run_id
          and artifact['workflow_run'].get('head_sha') == sha and 0 < artifact['size_in_bytes'] <= 65536, 'Pending receipt origin differs')
    functional.entry.execution.upload_binding(job, UPLOAD, artifact)
    receipt = functional.exact_plan(transport.archive(artifact['id']), artifact, 'release-functional-pending.json')
    commit = transport.get('/git/commits/' + sha)
    check(receipt == functional.pending_record({'sha': sha, 'tree': commit['tree']['sha']}, {'runId': run_id, 'attempt': attempt}), 'Pending receipt source or intent differs')
    functional.retry.verify_unfrozen_retry(transport.get, run_id=run_id, source_sha=sha, current_attempt=attempt + 1,
        workflow=WORKFLOW, producer=functional.entry.PRODUCER, freeze_step='Freeze independent release candidates',
        publisher='Publish exact independent release archives under durable lock', events=['push'])
    return {'action': 'rerun-failed-jobs', 'runId': run_id, 'attempt': attempt, 'sourceSHA': sha, 'functional': accepted}


def recover(transport, event):
    first = decision(transport, event)
    if first['action'] != 'rerun-failed-jobs': return first
    # Re-read immediately before the only write. Concurrent/manual reruns become no-ops.
    second = decision(transport, event)
    if second != first: return {'action': 'none', 'reason': 'recovery-state-changed'}
    # YAML preserves the original plan and candidate attempt outputs across carries.
    transport.rerun(first['runId'])
    return {**first, 'requested': True, 'publicationExecuted': False}


class Actions:
    def get(self, route):
        check(route.startswith('/') and '..' not in route, 'Unexpected recovery route')
        return json.loads(subprocess.check_output(['gh', 'api', 'repos/' + REPOSITORY + route], timeout=60))

    def archive(self, artifact_id):
        check(type(artifact_id) is int and artifact_id > 0, 'Invalid artifact ID')
        body = subprocess.check_output(['gh', 'api', 'repos/' + REPOSITORY + '/actions/artifacts/' + str(artifact_id) + '/zip'], timeout=60)
        check(len(body) <= 65536, 'Pending archive exceeds bound'); return body

    def rerun(self, run_id):
        check(type(run_id) is int and run_id > 0, 'Invalid rerun ID')
        subprocess.run(['gh', 'api', '--method', 'POST', 'repos/' + REPOSITORY + '/actions/runs/' + str(run_id) + '/rerun-failed-jobs'], check=True, timeout=60)


if __name__ == '__main__':
    check(os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_run'
          and os.environ.get('GITHUB_REPOSITORY') == REPOSITORY and os.environ.get('GITHUB_REPOSITORY_ID') == str(REPOSITORY_ID)
          and os.environ.get('GITHUB_WORKFLOW_REF') == REPOSITORY + '/' + RECOVERY + '@refs/heads/main', 'Only trusted main recovery workflow may request retries')
    check(subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == os.environ['GITHUB_WORKFLOW_SHA'], 'Recovery control checkout differs')
    print(json.dumps(recover(Actions(), json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text())), sort_keys=True))
