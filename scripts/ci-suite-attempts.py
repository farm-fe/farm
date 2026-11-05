"""Read-only same-run receipt resolution. Never rebuild or rename a producer artifact."""
import argparse
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path

spec = importlib.util.spec_from_file_location('rerun_suite_execution', Path(__file__).with_name('ci-execution-evidence.py'))
execution = importlib.util.module_from_spec(spec); spec.loader.exec_module(execution)
check = execution.check


def original_selection(transport, run, source, selection_hash):
    producer, job = execution.actual_producer(transport, run, 'Type Check')
    attempt = producer['run_attempt']
    selection, artifact = execution.fetch_json_artifact(transport, producer, 'ci-suite-selection-' + str(run['id']) + '-' + str(attempt), 'selection.json')
    execution.upload_binding(job, 'Upload bound current suite selection', artifact)
    import io, zipfile
    with zipfile.ZipFile(io.BytesIO(transport.archive(artifact['id']))) as archive: body = archive.read('selection.json')
    check(hashlib.sha256(body).hexdigest() == selection_hash, 'Original selection bytes differ from trusted output')
    check(selection.get('sourceSHA') == source['sha'] and selection.get('sourceTree') == source['tree'] and selection.get('runId') == run['id'] and selection.get('attempt') == attempt and selection.get('repositoryId') == run['repository']['id'], 'Original selection source or attempt differs')
    return attempt, body


def restore_selection(transport, run, source, selection_hash, output):
    attempt, body = original_selection(transport, run, source, selection_hash)
    path = Path(output); check(not path.exists(), 'Refuse stale selection destination')
    path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(body)
    return attempt


def collect(transport, run, source, output, selection_hash='', selection_output=None, upload_outputs=None):
    check(run['path'] == '.github/workflows/ci.yaml' and run['event'] in ['push', 'pull_request', 'workflow_dispatch'], 'Only CI receipts may be resolved')
    root = Path(output); check(not root.exists(), 'Refuse stale rerun receipt directory')
    rows = []; attempts = {}
    for abi in execution.PLATFORMS:
        for kind in ['ts', 'examples-browser']:
            data, artifact, binding = execution.suite_receipt(transport, run, source, kind, abi, cache=attempts, trusted_output=None if upload_outputs is None else upload_outputs[abi][kind])
            rows.append((data, artifact, binding))
    selection_attempt = None
    selection = None
    if selection_hash:
        selection_attempt, body = original_selection(transport, run, source, selection_hash)
        selection = True
        check(selection_output is not None and not Path(selection_output).exists(), 'Refuse stale or absent selection destination')
    root.mkdir(parents=True)
    proofs = {}
    for data, artifact, binding in rows:
        folder = root / artifact['name']; folder.mkdir()
        filename = 'ts.json' if data['kind'] == 'ts' else 'browser.json'
        (folder / filename).write_text(json.dumps(data), encoding='utf-8')
        proofs[artifact['name']] = binding
    if selection is not None:
        destination = Path(selection_output); destination.parent.mkdir(parents=True, exist_ok=True); destination.write_bytes(body)
    (root / 'producers.json').write_text(json.dumps({'schemaVersion':1, 'sourceSHA':source['sha'], 'sourceTree':source['tree'], 'runId':run['id'], 'aggregationAttempt':run['run_attempt'], 'selectionAttempt':selection_attempt, 'producers':proofs}), encoding='utf-8')
    return proofs



def current_upload_outputs(environment):
    """Read only fixed workflow needs outputs, never an artifact-supplied binding."""
    result = {}
    fields = {'artifact_id': r'[1-9][0-9]*', 'artifact_digest': r'[a-f0-9]{64}', 'run_attempt': r'[1-9][0-9]*'}
    for abi, platform in [('linux-x64-gnu','LINUX'), ('darwin-arm64','MACOS'), ('win32-x64-msvc','WINDOWS')]:
        row = json.loads(environment['FARM_SUITE_UPLOADS_' + platform])
        suite_fields = {kind + '_' + field for kind in ['ts','browser'] for field in fields}
        shadow_fields = {kind + '_shadow_' + field for kind in ['ts','browser'] for field in fields}
        # Accept only the two reviewed producer interfaces. Older carried jobs
        # export six suite fields; current jobs additionally export all six shadow fields.
        check(isinstance(row, dict) and set(row) in [suite_fields, suite_fields | shadow_fields], 'Missing or unexpected suite upload outputs')
        if set(row) == suite_fields | shadow_fields:
            check(all(isinstance(row[label + '_shadow_' + field], str) and re.fullmatch(pattern, row[label + '_shadow_' + field])
                      for label in ['ts','browser'] for field, pattern in fields.items()), 'Malformed shadow upload outputs')
        result[abi] = {}
        for label, kind in [('ts','ts'), ('browser','examples-browser')]:
            output = {field: row[label + '_' + field] for field in fields}
            check(all(isinstance(output[field], str) and re.fullmatch(pattern, output[field]) for field, pattern in fields.items()), 'Malformed suite upload outputs')
            result[abi][kind] = output
    return result


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--output'); parser.add_argument('--selection-only', action='store_true'); parser.add_argument('--selection-output', default='ci-suite-selection/selection.json'); args = parser.parse_args()
    env = os.environ
    check(env.get('GITHUB_ACTIONS') == 'true' and env.get('GITHUB_JOB') in ['performance-summary', 'incremental-execution-gate', 'ts-test', 'examples-test'], 'Unexpected receipt resolver job')
    transport = execution.evidence.transport_module.GitHubReadOnly(env['GITHUB_REPOSITORY'], env['GH_TOKEN'], allow_archive=True)
    run = transport.get('/actions/runs/' + env['GITHUB_RUN_ID'] + '/attempts/' + env['GITHUB_RUN_ATTEMPT'])
    check(run['id'] == int(env['GITHUB_RUN_ID']) and run['run_attempt'] == int(env['GITHUB_RUN_ATTEMPT']) and run['repository']['id'] == int(env['GITHUB_REPOSITORY_ID']) == run['head_repository']['id'], 'Resolver executing run differs')
    source = execution.evidence.source_reader(env['GITHUB_SHA'])
    # PR Actions checkout may be a merge SHA while head_sha is its head parent.
    check(source['sha'] == run['head_sha'] or run['event'] == 'pull_request' and run['head_sha'] in source['parents'], 'Resolver source differs from fixed run')
    if args.selection_only:
        attempt = restore_selection(transport, run, source, env['FARM_SUITE_SELECTION_HASH'], args.selection_output)
        with Path(env['GITHUB_ENV']).open('a', encoding='utf-8') as stream: stream.write('CI_SUITE_SELECTION_PRODUCER_ATTEMPT=' + str(attempt) + '\n')
        print(json.dumps({'selectionProducerAttempt':attempt, 'currentConsumerAttempt':run['run_attempt']})); return
    check(args.output is not None and env.get('GITHUB_JOB') in ['performance-summary','incremental-execution-gate'], 'Suite collection role or output missing')
    print(json.dumps({'cells':len(collect(transport, run, source, args.output, env.get('FARM_SUITE_SELECTION_HASH', ''), args.selection_output, current_upload_outputs(env))), 'buildExecuted':False}))

if __name__ == '__main__': main()
