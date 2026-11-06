"""Read-only mixed-attempt Unit observations; no acceptance or publication grant."""
# cspell:words importlib
import importlib.util
import json
import os
import re
from pathlib import Path
import zipfile

spec = importlib.util.spec_from_file_location('unit_shadow_execution', Path(__file__).with_name('ci-execution-evidence.py'))
execution = importlib.util.module_from_spec(spec); spec.loader.exec_module(execution)
check = execution.check
impact = execution.evidence.transport_module.impact
CELLS = [('cargo-check', 'Check', 'cargo-check', 'single'),
         *[('rust-tests:ci-test:' + abi, 'Test - ' + platform, 'cargo-test', platform)
           for abi, platform in execution.PLATFORMS.items()]]


def unit_upload_outputs(environment):
    """Fixed per-job needs outputs; no matrix last-writer or receipt assertions."""
    values = json.loads(environment['CI_UNIT_UPLOADS'])
    check(isinstance(values, dict) and set(values) == {'planner', *[cell for cell, *_ in CELLS]}, 'Unit upload output inventory differs')
    patterns = {'artifact_id': r'[1-9][0-9]*', 'artifact_digest': r'[a-f0-9]{64}', 'run_attempt': r'[1-9][0-9]*'}
    for row in values.values():
        check(isinstance(row, dict) and set(row) == set(patterns), 'Missing or unexpected Unit upload outputs')
        check(all(isinstance(row[field], str) and re.fullmatch(pattern, row[field]) for field, pattern in patterns.items()), 'Malformed Unit upload outputs')
    return values['planner'], {cell: values[cell] for cell, *_ in CELLS}


def read_object(transport, run, name, member):
    try:
        value, artifact = execution.fetch_json_artifact(transport, run, name, member)
    except zipfile.BadZipFile as error:
        raise ValueError('Unit shadow artifact ZIP is invalid') from error
    check(isinstance(value, dict), 'Unit shadow JSON must be an object')
    return value, artifact


def resolve(transport, run, identity, *, scope="unit", cells=None, upload_outputs=None, planner_output=None, require_explicit=False):
    check(scope in ["unit", "e2e"], "Unexpected shadow scope")
    workflow = "rust-test.yaml" if scope == "unit" else "ci.yaml"
    planner = "Incremental Unit CI plan (shadow only)" if scope == "unit" else "Incremental CI plan (shadow only)"
    cells = CELLS if scope == "unit" else cells
    check(cells is not None and sorted(row[0] for row in cells) == sorted(impact.execution_cells(scope)), "Shadow producer inventory differs")
    check(run['path'] == '.github/workflows/' + workflow and run['event'] in ['pull_request','push','workflow_dispatch'], 'Unit shadow workflow differs')
    check(run['repository']['id'] == run['head_repository']['id'] and run['repository']['full_name'] == identity['repository']
          and str(run['id']) == identity['runId'] and run['head_sha'] == identity['headSHA'], 'Unit shadow run source differs')
    if require_explicit:
        check(isinstance(planner_output, dict) and isinstance(upload_outputs, dict)
              and set(upload_outputs) == {row[0] for row in cells}
              and all(isinstance(row, dict) for row in upload_outputs.values()), 'Explicit shadow producer bindings required')
    cache = {}
    context = {'resolutionStep':'resolve-producer'}
    try:
        planner_run, planner_job = (execution.explicit_producer(transport, run, planner, planner_output, cache=cache)
                                    if (scope == 'unit' or require_explicit) and planner_output is not None
                                    else execution.actual_producer(transport, run, planner, cache=cache))
        context.update(producerAttempt=planner_run['run_attempt'], producerJobId=planner_job['id'], resolutionStep='read-artifact')
        name = 'impact-plan-' + identity['testedSHA'] + '-' + str(planner_run['run_attempt'])
        plan, artifact = read_object(transport, planner_run, name, 'plan.json')
        context.update(artifactId=artifact['id'], resolutionStep='bind-upload')
        if planner_output is not None:
            check(planner_output['run_attempt'] == str(planner_run['run_attempt']), 'Planner upload output differs from original producer attempt')
        execution.upload_binding(planner_job, 'Upload fixed source impact plan', artifact, planner_output)
        context['resolutionStep'] = 'validate-digest'
        check(plan.get('planHash') == impact.digest({key:value for key,value in plan.items() if key != 'planHash'}), 'Unit shadow plan digest differs')
        context['resolutionStep'] = 'validate-source'
        for key,value in identity.items():
            context['identityField'] = key
            check(plan.get(key) == value, 'Unit shadow plan source differs: ' + key)
        context.pop('identityField', None)
        check(plan.get('runAttempt') == str(planner_run['run_attempt']) and plan.get('mode') == 'shadow'
              and plan.get('skipAuthorized') is False and plan.get('reuseAuthorized') is False, 'Unit shadow plan authority or attempt differs')
        check(plan.get('executionCells') == impact.execution_cells(scope), 'Unit shadow inventory differs')
    except (OSError, ValueError, KeyError) as error:
        # An unverified plan must never become shadow_plan: the summary must
        # remain planUnavailable and cannot bind observations after this error.
        reasons = {
            'Unproven carried suite producer': 'carried-producer-unproven',
            'Latest actual suite producer did not succeed': 'latest-producer-unsuccessful',
            'Exact execution artifact missing or ambiguous': 'artifact-missing-or-ambiguous',
            'Execution artifact bytes differ': 'artifact-bytes-differ',
            'Execution receipt members differ': 'artifact-members-differ',
            'Actual producer upload did not succeed': 'upload-step-unsuccessful',
            'Artifact is outside its fixed upload step: ': 'upload-time-window',
        }
        context['resolutionReason'] = next((reason for prefix,reason in reasons.items() if str(error).startswith(prefix)), 'validation-rejected')
        error.shadow_context = context
        raise
    try:
        observations, bindings = [], {}
        for cell, producer, job_key, suffix in cells:
            context = {'cell':cell, 'resolutionStep':'resolve-producer'}
            output = None if upload_outputs is None else upload_outputs.get(cell)
            origin, job = (execution.explicit_producer(transport, run, producer, output, cache=cache)
                           if (scope == 'unit' or require_explicit) and output is not None
                           else execution.actual_producer(transport, run, producer, cache=cache))
            context.update(producerAttempt=origin['run_attempt'], producerJobId=job['id'], resolutionStep='read-artifact')
            name = 'impact-cell-' + identity['testedSHA'] + '-' + job_key + '-' + suffix + '-' + str(origin['run_attempt'])
            row, meta = read_object(transport, origin, name, 'receipt.json')
            context.update(artifactId=meta['id'], resolutionStep='bind-upload')
            output = None if upload_outputs is None else upload_outputs.get(cell)
            if output is not None:
                check(output['run_attempt'] == str(origin['run_attempt']), 'Shadow upload output differs from original producer attempt')
            execution.upload_binding(job, 'Preserve actual cell receipt (shadow only)', meta, output)
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


def current(environment, *, scope="unit", cells=None, upload_outputs=None, planner_output=None):
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
    if scope == 'unit':
        planner_output, upload_outputs = unit_upload_outputs(environment)
    return resolve(transport, run, identity, scope=scope, cells=cells, upload_outputs=upload_outputs, planner_output=planner_output, require_explicit=True)
