"""Restore same-run functional evidence using fixed upload outputs, never clocks."""
# cspell:words importlib
import argparse
import importlib.util
import json
import os
from pathlib import Path


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


execution = load('functional_upload_execution', 'ci-execution-evidence.py')
functional = load('functional_upload_policy', 'incremental-release-functional.py')
check = execution.check
PRODUCER = 'Verify actual fixed-main functional evidence'
UPLOAD = 'Preserve verified fixed-main functional evidence'


def restore(transport, run, binding):
    check(run['path'] == '.github/workflows/incremental-release.yaml'
          and run['repository']['id'] == run['head_repository']['id']
          and run['repository']['full_name'] == 'farm-fe/farm'
          and run['head_branch'] == 'main' and run['event'] in ['push', 'workflow_dispatch'],
          'Functional evidence run identity differs')
    origin, job = execution.explicit_producer(transport, run, PRODUCER, binding)
    artifact = transport.get('/actions/artifacts/' + binding['artifact_id'])
    check(artifact['name'] == 'release-functional-' + str(run['id']) + '-' + str(origin['run_attempt'])
          and artifact['workflow_run']['id'] == run['id']
          and artifact['workflow_run']['head_sha'] == run['head_sha']
          and artifact.get('expired') is False, 'Functional evidence artifact origin differs')
    execution.upload_binding(job, UPLOAD, artifact, binding)
    return functional.exact_plan(transport.archive(artifact['id']), artifact, 'functional-evidence.json')


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--output', required=True); args = parser.parse_args()
    env = os.environ
    check(env.get('GITHUB_ACTIONS') == 'true' and env.get('GITHUB_JOB') == 'producer'
          and env.get('GITHUB_REPOSITORY') == 'farm-fe/farm', 'Unexpected functional evidence consumer')
    transport = execution.evidence.transport_module.GitHubReadOnly(env['GITHUB_REPOSITORY'], env['GH_TOKEN'], allow_archive=True)
    run = transport.get('/actions/runs/' + env['GITHUB_RUN_ID'] + '/attempts/' + env['GITHUB_RUN_ATTEMPT'])
    check(run['id'] == int(env['GITHUB_RUN_ID']) and run['run_attempt'] == int(env['GITHUB_RUN_ATTEMPT'])
          and run['head_sha'] == env['GITHUB_SHA'] and str(run['repository']['id']) == env['GITHUB_REPOSITORY_ID'],
          'Functional evidence executing source differs')
    body = restore(transport, run, json.loads(env['FARM_FUNCTIONAL_UPLOAD']))
    destination = Path(args.output); check(not destination.exists(), 'Refuse stale functional evidence destination')
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(body, sort_keys=True) + '\n', encoding='utf-8')


if __name__ == '__main__': main()
