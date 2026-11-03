"""Read-only mixed-attempt Unit observations; no acceptance or publication grant."""
# cspell:words importlib
import importlib.util
import json
import os
from pathlib import Path
import zipfile

spec = importlib.util.spec_from_file_location('unit_shadow_execution', Path(__file__).with_name('ci-execution-evidence.py'))
execution = importlib.util.module_from_spec(spec); spec.loader.exec_module(execution)
check = execution.check
impact = execution.evidence.transport_module.impact
CELLS = [('cargo-check', 'Check', 'cargo-check', 'single'),
         *[('rust-tests:ci-test:' + abi, 'Test - ' + platform, 'cargo-test', platform)
           for abi, platform in execution.PLATFORMS.items()]]


def read_object(transport, run, name, member):
    try:
        value, artifact = execution.fetch_json_artifact(transport, run, name, member)
    except zipfile.BadZipFile as error:
        raise ValueError('Unit shadow artifact ZIP is invalid') from error
    check(isinstance(value, dict), 'Unit shadow JSON must be an object')
    return value, artifact


def resolve(transport, run, identity, *, scope="unit", cells=None):
    check(scope in ["unit", "e2e"], "Unexpected shadow scope")
    workflow = "rust-test.yaml" if scope == "unit" else "ci.yaml"
    planner = "Incremental Unit CI plan (shadow only)" if scope == "unit" else "Incremental CI plan (shadow only)"
    cells = CELLS if scope == "unit" else cells
    check(cells is not None and sorted(row[0] for row in cells) == sorted(impact.execution_cells(scope)), "Shadow producer inventory differs")
    check(run['path'] == '.github/workflows/' + workflow and run['event'] in ['pull_request','push','workflow_dispatch'], 'Unit shadow workflow differs')
    check(run['repository']['id'] == run['head_repository']['id'] and run['repository']['full_name'] == identity['repository']
          and str(run['id']) == identity['runId'] and run['head_sha'] == identity['headSHA'], 'Unit shadow run source differs')
    cache = {}
    planner_run, planner_job = execution.actual_producer(transport, run, planner, cache=cache)
    name = 'impact-plan-' + identity['testedSHA'] + '-' + str(planner_run['run_attempt'])
    plan, artifact = read_object(transport, planner_run, name, 'plan.json')
    execution.upload_binding(planner_job, 'Upload fixed source impact plan', artifact)
    check(plan.get('planHash') == impact.digest({key:value for key,value in plan.items() if key != 'planHash'}), 'Unit shadow plan digest differs')
    for key,value in identity.items(): check(plan.get(key) == value, 'Unit shadow plan source differs: ' + key)
    check(plan.get('runAttempt') == str(planner_run['run_attempt']) and plan.get('mode') == 'shadow'
          and plan.get('skipAuthorized') is False and plan.get('reuseAuthorized') is False, 'Unit shadow plan authority or attempt differs')
    check(plan.get('executionCells') == impact.execution_cells(scope), 'Unit shadow inventory differs')
    try:
        observations, bindings = [], {}
        for cell, producer, job_key, suffix in cells:
            context = {'cell':cell, 'resolutionStep':'resolve-producer'}
            origin, job = execution.actual_producer(transport, run, producer, cache=cache)
            context.update(producerAttempt=origin['run_attempt'], producerJobId=job['id'], resolutionStep='read-artifact')
            name = 'impact-cell-' + identity['testedSHA'] + '-' + job_key + '-' + suffix + '-' + str(origin['run_attempt'])
            row, meta = read_object(transport, origin, name, 'receipt.json')
            context.update(artifactId=meta['id'], resolutionStep='bind-upload')
            execution.upload_binding(job, 'Preserve actual cell receipt (shadow only)', meta)
            context['resolutionStep'] = 'validate-digest'
            check(row.get('observationHash') == impact.digest({key:value for key,value in row.items() if key != 'observationHash'}), 'Unit observation digest differs')
            context['resolutionStep'] = 'validate-source'
            for key,value in identity.items():
                context['identityField'] = key
                check(row.get(key) == value, 'Unit observation source differs: ' + key)
            context.pop('identityField', None)
            context['resolutionStep'] = 'validate-cell'
            check(row.get('schemaVersion') == 2 and row.get('cell') == cell and row.get('jobKey') == job_key
                  and row.get('runAttempt') == str(origin['run_attempt']) and row.get('status') == 'executed-success'
                  and row.get('planBound') is False and row.get('acceptanceInherited') is False
                  and row.get('authoritativeJobIdentityVerified') is False, 'Unit observation identity or authority differs')
            context['resolutionStep'] = 'validate-workflow'
            check(row.get('workflowRef', '').startswith(identity['repository'] + '/.github/workflows/' + workflow + '@'), 'Unit observation workflow differs')
            observations.append(row)
            bindings[cell] = {'producerAttempt':origin['run_attempt'], 'producerJobId':job['id'], 'artifactId':meta['id'], 'artifactDigest':meta['digest']}
    except (OSError, ValueError, KeyError) as error:
        error.shadow_plan = plan
        reasons = {
            'Artifact is outside its fixed upload step: ': 'upload-time-window',
            'Actual producer upload did not succeed': 'upload-step-unsuccessful',
            'Exact execution artifact missing or ambiguous': 'artifact-missing-or-ambiguous',
            'Execution artifact bytes differ': 'artifact-bytes-differ',
            'Execution receipt members differ': 'artifact-members-differ',
            'Latest actual suite producer did not succeed': 'latest-producer-unsuccessful',
            'Unproven carried suite producer': 'carried-producer-unproven',
        }
        context['resolutionReason'] = next((reason for prefix,reason in reasons.items() if str(error).startswith(prefix)), 'validation-rejected')
        error.shadow_context = context
        raise
    return plan, observations, {'plannerAttempt':planner_run['run_attempt'], 'plannerJobId':planner_job['id'],
                                'planArtifactId':artifact['id'], 'planArtifactDigest':artifact['digest'],
                                'consumerAttempt':run['run_attempt'], 'cells':bindings}


def current(environment, *, scope="unit", cells=None):
    check(environment.get('GITHUB_ACTIONS') == 'true' and environment.get('GITHUB_JOB') == 'impact-summary'
          and environment.get('CI_IMPACT_SCOPE') == scope, 'Unexpected Unit shadow consumer')
    event = json.loads(Path(environment['GITHUB_EVENT_PATH']).read_text(encoding='utf-8'))
    pr = event.get('pull_request', {})
    sha = impact.git('rev-parse','HEAD').decode('ascii').strip()
    check(sha == environment['GITHUB_SHA'], 'Unit summary checkout differs')
    identity = {'headSHA':pr.get('head',{}).get('sha') or sha,
                'baseSHA':pr.get('base',{}).get('sha') or event.get('inputs',{}).get('base_sha') or event.get('inputs',{}).get('release_base') or event.get('before'),
                'testedSHA':sha, 'testedTree':impact.git('rev-parse','HEAD^{tree}').decode('ascii').strip(),
                'runId':environment['GITHUB_RUN_ID'], 'repository':environment['GITHUB_REPOSITORY'], 'policy':impact.POLICY}
    check(all(isinstance(value,str) and value for value in identity.values()), 'Unit summary identity unavailable')
    transport = execution.evidence.transport_module.GitHubReadOnly(identity['repository'], environment['GH_TOKEN'], allow_archive=True)
    run = transport.get('/actions/runs/' + identity['runId'] + '/attempts/' + environment['GITHUB_RUN_ATTEMPT'])
    check(str(run['run_attempt']) == environment['GITHUB_RUN_ATTEMPT']
          and str(run['repository']['id']) == environment['GITHUB_REPOSITORY_ID'], 'Unit summary current attempt differs')
    return resolve(transport, run, identity, scope=scope, cells=cells)
