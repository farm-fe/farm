"""Policy for a verified cross-run binding manifest. Not yet wired to Actions.

The verification result must come directly from successful pinned gh attestation
verification, never from an artifact or caller-supplied JSON. This policy does not
perform cryptography or authorize unsigned input. Tests explicitly model that
external verification boundary; no signing or token permissions are enabled.
"""
import hashlib
import json
import re

PREDICATE = 'https://farmfe.org/attestations/ci-bindings/v1'


def check(condition, message):
    if not condition:
        raise ValueError(message)


def strict_json(body):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            check(key not in result, 'Duplicate manifest key')
            result[key] = value
        return result
    check(isinstance(body, bytes) and 0 < len(body) <= 4 * 1024 * 1024, 'Manifest size differs')
    return json.loads(body.decode('utf-8'), object_pairs_hook=unique)


def verify_caller_templates(files, templates, signer_revision):
    """Exact reviewed source bytes; one explicit signer revision placeholder."""
    check(re.fullmatch(r'[a-f0-9]{40}', signer_revision) is not None, 'Invalid signer revision')
    check(set(files) == set(templates) and files, 'Caller template inventory differs')
    placeholder = b'__REVIEWED_SIGNER_REVISION__'
    for path, template in templates.items():
        check(isinstance(template, bytes) and isinstance(files[path], bytes), 'Template bytes required')
        check(template.count(placeholder) <= 1, 'Ambiguous signer placeholder')
        expected = template.replace(placeholder, signer_revision.encode())
        check(files[path] == expected, 'Caller or producer template differs: ' + path)


def validate_verified_manifest(body, verification, expected):
    """Additional policy AFTER cryptographic verification; no fallback claims."""
    manifest = strict_json(body)
    check(isinstance(verification, list) and len(verification) == 1, 'Missing or ambiguous verified attestation')
    result = verification[0].get('verificationResult', {})
    certificate = result.get('signature', {}).get('certificate', {})
    repo = 'https://github.com/' + expected['repository']
    signer = repo + '/' + expected['signerWorkflow'] + '@' + expected['signerSHA']
    cert = {
        'issuer':'https://token.actions.githubusercontent.com',
        'subjectAlternativeName':signer,
        'buildSignerURI':signer,
        'buildSignerDigest':expected['signerSHA'],
        'runnerEnvironment':'github-hosted',
        'sourceRepositoryURI':repo,
        'sourceRepositoryIdentifier':str(expected['repositoryId']),
        'sourceRepositoryDigest':expected['sourceSHA'],
        'sourceRepositoryRef':'refs/heads/main',
        'buildConfigURI':repo + '/' + expected['callerWorkflow'] + '@refs/heads/main',
        'buildConfigDigest':expected['sourceSHA'],
        'buildTrigger':'push',
        'runInvocationURI':repo + '/actions/runs/' + str(expected['runId']) + '/attempts/' + str(expected['attempt']),
    }
    check(all(certificate.get(key) == value for key, value in cert.items()), 'Verified certificate identity differs or is incomplete')
    statement = result.get('statement', {})
    check(statement.get('_type') == 'https://in-toto.io/Statement/v1' and statement.get('predicateType') == PREDICATE, 'Binding statement type differs')
    check(statement.get('subject') == [{'name':'ci-bindings.json', 'digest':{'sha256':hashlib.sha256(body).hexdigest()}}], 'Verified manifest subject differs')
    check(statement.get('predicate') == {'schemaVersion':1, 'kind':'fixed-workflow-upload-bindings'}, 'Binding predicate differs')
    fields = {'schemaVersion','kind','repositoryId','sourceSHA','sourceTree','runId','aggregationAttempt','callerWorkflow','bindings'}
    check(isinstance(manifest, dict) and set(manifest) == fields, 'Manifest schema differs')
    check(manifest['schemaVersion'] == 1 and manifest['kind'] == 'fixed-workflow-upload-bindings', 'Manifest kind differs')
    for key in ['repositoryId','sourceSHA','sourceTree','runId','callerWorkflow']:
        check(manifest[key] == expected[key], 'Manifest identity differs: ' + key)
    check(type(manifest['aggregationAttempt']) is int and manifest['aggregationAttempt'] == expected['attempt'], 'Manifest aggregation attempt differs')
    rows = manifest['bindings']
    check(isinstance(rows, list) and len(rows) == len(expected['cells']), 'Binding inventory count differs')
    seen, artifact_ids, bound = set(), set(), {}
    for row in rows:
        keys = {'cell','producer','jobId','originAttempt','uploadStep','artifactId','artifactDigest','artifactName'}
        check(isinstance(row, dict) and set(row) == keys, 'Binding row schema differs')
        cell = row['cell']
        check(isinstance(cell, str) and cell in expected['cells'] and cell not in seen, 'Missing, duplicate or unknown cell')
        seen.add(cell); policy = expected['cells'][cell]
        check(row['producer'] == policy['producer'] and row['uploadStep'] == policy['uploadStep'], 'Binding producer role differs')
        check(type(row['originAttempt']) is int and 0 < row['originAttempt'] <= expected['attempt'], 'Binding original attempt differs')
        check(type(row['jobId']) is int and row['jobId'] > 0 and type(row['artifactId']) is int and row['artifactId'] > 0, 'Invalid binding IDs')
        check(row['artifactId'] not in artifact_ids, 'Artifact reused for multiple cells');artifact_ids.add(row['artifactId'])
        check(isinstance(row['artifactDigest'], str) and re.fullmatch(r'sha256:[a-f0-9]{64}', row['artifactDigest']), 'Invalid binding digest')
        name = policy['artifactName'].format(sourceSHA=expected['sourceSHA'], runId=expected['runId'], originAttempt=row['originAttempt'])
        check(row['artifactName'] == name, 'Binding artifact namespace differs')
        bound[cell] = dict(row)
    check(seen == set(expected['cells']), 'Incomplete binding inventory')
    return bound


def verify_with_pinned_cli(cli, cli_sha256, manifest_file, bundle_file, expected, *, invoke=None):
    """Read-only verifier seam; no caller-provided verified-result file is accepted.

    Deployment must provide a reviewed CLI digest and signer policy from trusted
    configuration, not the downloaded manifest. Offline tests replace only the
    subprocess boundary, and are not cryptographic acceptance.
    """
    import os
    from pathlib import Path
    import subprocess
    cli, manifest_file, bundle_file = map(Path, [cli, manifest_file, bundle_file])
    check(isinstance(cli_sha256, str) and re.fullmatch(r'[a-f0-9]{64}', cli_sha256), 'Reviewed CLI digest required')
    check(cli.is_file() and not cli.is_symlink() and hashlib.sha256(cli.read_bytes()).hexdigest() == cli_sha256, 'Verification executable differs')
    check(manifest_file.is_file() and not manifest_file.is_symlink() and 0 < manifest_file.stat().st_size <= 4 * 1024 * 1024, 'Invalid manifest file')
    check(bundle_file.is_file() and not bundle_file.is_symlink() and 0 < bundle_file.stat().st_size <= 16 * 1024 * 1024, 'Invalid attestation bundle')
    body = manifest_file.read_bytes()
    signer = 'https://github.com/' + expected['repository'] + '/' + expected['signerWorkflow'] + '@' + expected['signerSHA']
    # Exact identity already binds the workflow and revision; gh forbids combining
    # this selector with --signer-workflow. Keep all independent digest checks.
    command = [str(cli.resolve()), 'attestation', 'verify', str(manifest_file.resolve()), '--bundle', str(bundle_file.resolve()),
               '--repo', expected['repository'], '--cert-identity', signer,
               '--signer-digest', expected['signerSHA'], '--source-digest', expected['sourceSHA'], '--source-ref', 'refs/heads/main',
               '--predicate-type', PREDICATE, '--cert-oidc-issuer', 'https://token.actions.githubusercontent.com', '--deny-self-hosted-runners', '--format', 'json']
    # The verifier reads local files and public trust material, never signs or publishes.
    import tempfile
    with tempfile.TemporaryDirectory(prefix='farm-verifier-home-') as home:
        env = {'PATH':os.defpath, 'HOME':home, 'GH_CONFIG_DIR':str(Path(home) / 'config'),
               'GH_PROMPT_DISABLED':'1', 'GH_NO_UPDATE_NOTIFIER':'1', 'LANG':'C.UTF-8'}
        result = (subprocess.run if invoke is None else invoke)(command, capture_output=True, timeout=120, check=False, env=env)
    check(result.returncode == 0, 'Cryptographic attestation verification failed')
    check(isinstance(result.stdout, bytes) and 0 < len(result.stdout) <= 16 * 1024 * 1024, 'Invalid verifier response')
    check(manifest_file.read_bytes() == body, 'Manifest changed during verification')
    return validate_verified_manifest(body, strict_json(result.stdout), expected)


def build_manifest_from_fixed_outputs(transport, expected, outputs, templates):
    """Signer-side pure orchestration; caller sources are fetched at fixed SHA.

    `expected` and `templates` belong to the reviewed signer revision. `outputs`
    must come from its exact authenticated caller wiring, never a downloaded
    manifest. This function does not sign and is not enabled in any workflow.
    """
    import base64
    import importlib.util
    from pathlib import Path
    from urllib.parse import quote
    spec = importlib.util.spec_from_file_location('attested_origin', Path(__file__).with_name('ci-execution-evidence.py'))
    execution = importlib.util.module_from_spec(spec); spec.loader.exec_module(execution)
    run = transport.get('/actions/runs/' + str(expected['runId']) + '/attempts/' + str(expected['attempt']))
    check(run.get('id') == expected['runId'] and run.get('run_attempt') == expected['attempt']
          and run.get('repository', {}).get('id') == expected['repositoryId']
          and run.get('head_repository', {}).get('id') == expected['repositoryId']
          and run.get('repository', {}).get('full_name') == expected['repository']
          and run.get('head_sha') == expected['sourceSHA'] and run.get('head_branch') == 'main'
          and run.get('event') == 'push' and run.get('path') == expected['callerWorkflow'], 'Signer caller run identity differs')
    commit = transport.get('/git/commits/' + expected['sourceSHA'])
    check(commit.get('sha') == expected['sourceSHA'] and commit.get('tree', {}).get('sha') == expected['sourceTree'], 'Signer caller source tree differs')
    files = {}
    check(expected['callerWorkflow'] in templates, 'Caller workflow template missing')
    for path in templates:
        check(isinstance(path, str) and path.startswith(('.github/', 'scripts/')) and '..' not in path.split('/'), 'Invalid caller template path')
        item = transport.get('/contents/' + quote(path, safe='/') + '?ref=' + expected['sourceSHA'])
        check(item.get('type') == 'file' and item.get('path') == path and item.get('encoding') == 'base64'
              and type(item.get('size')) is int and 0 < item['size'] <= 1024 * 1024, 'Caller template source missing or invalid')
        content = base64.b64decode(''.join(item.get('content', '').split()), validate=True)
        check(len(content) == item['size'], 'Caller template source size differs');files[path] = content
    verify_caller_templates(files, templates, expected['signerSHA'])
    check(isinstance(outputs, dict) and set(outputs) == set(expected['cells']), 'Fixed caller output inventory differs')
    rows, cache = [], {}
    for cell, role in expected['cells'].items():
        output = outputs[cell]
        origin, job = execution.explicit_producer(transport, run, role['producer'], output, cache, required_conclusion=role.get('producerConclusion', 'success'))
        artifact = transport.get('/actions/artifacts/' + output['artifact_id'])
        name = role['artifactName'].format(sourceSHA=expected['sourceSHA'], runId=expected['runId'], originAttempt=origin['run_attempt'])
        check(artifact.get('name') == name and artifact.get('workflow_run', {}).get('id') == run['id']
              and artifact.get('expired') is False, 'Signer artifact origin differs')
        execution.upload_binding(job, role['uploadStep'], artifact, output)
        rows.append({'cell':cell,'producer':role['producer'],'jobId':job['id'],'originAttempt':origin['run_attempt'],
                     'uploadStep':role['uploadStep'],'artifactId':artifact['id'],'artifactDigest':artifact['digest'],'artifactName':name})
    manifest = {'schemaVersion':1,'kind':'fixed-workflow-upload-bindings',
                **{key:expected[key] for key in ['repositoryId','sourceSHA','sourceTree','runId','callerWorkflow']},
                'aggregationAttempt':expected['attempt'],'bindings':rows}
    return json.dumps(manifest, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()


def bound_artifact_metadata(transport, run, row, expected, *, maximum_bytes=4 * 1024 * 1024, required_conclusion="success"):
    """Resolve a row ONLY after validate_verified_manifest/verified CLI policy.

    API state is checked again, so a newer failed attempt cannot inherit success.
    The artifact is selected by signed ID, never a mutable discovery name or time.
    This returns archive bytes, not an authorization to execute their contents.
    """
    import importlib.util
    from pathlib import Path
    spec = importlib.util.spec_from_file_location('signed_execution', Path(__file__).with_name('ci-execution-evidence.py'))
    execution = importlib.util.module_from_spec(spec); spec.loader.exec_module(execution)
    # `expected` is the same exact policy used to verify the signed manifest.
    # An older signature is NOT a current needs output after another rerun.
    check(run.get('id') == expected['runId'] and run.get('run_attempt') == expected['attempt']
          and run.get('path') == expected['callerWorkflow'] and run.get('head_sha') == expected['sourceSHA']
          and run.get('repository', {}).get('id') == expected['repositoryId']
          and run.get('head_repository', {}).get('id') == expected['repositoryId']
          and run.get('event') == 'push' and run.get('head_branch') == 'main',
          'Signed aggregation run differs; a new attempt requires a new verified manifest')
    output = {'artifact_id':str(row['artifactId']), 'artifact_digest':row['artifactDigest'].removeprefix('sha256:'), 'run_attempt':str(row['originAttempt'])}
    origin, job = execution.explicit_producer(transport, run, row['producer'], output, required_conclusion=required_conclusion)
    check(job['id'] == row['jobId'] and origin['run_attempt'] == row['originAttempt'], 'Signed original producer differs')
    artifact = transport.get('/actions/artifacts/' + str(row['artifactId']))
    check(artifact.get('id') == row['artifactId'] and artifact.get('name') == row['artifactName']
          and artifact.get('workflow_run', {}).get('id') == run['id'] and artifact.get('expired') is False,
          'Signed artifact identity differs')
    check(type(artifact.get('size_in_bytes')) is int and 0 < artifact['size_in_bytes'] <= maximum_bytes, 'Signed archive bounds exceeded')
    execution.upload_binding(job, row['uploadStep'], artifact, output)
    return artifact, origin, job


def bound_artifact(transport, run, row, expected, *, maximum_bytes=4 * 1024 * 1024, required_conclusion="success"):
    artifact, origin, job = bound_artifact_metadata(transport, run, row, expected,
        maximum_bytes=maximum_bytes, required_conclusion=required_conclusion)
    body = transport.archive(row['artifactId'])
    check(isinstance(body, bytes) and len(body) == artifact['size_in_bytes']
          and 'sha256:' + hashlib.sha256(body).hexdigest() == row['artifactDigest'], 'Signed archive bytes differ')
    return body, artifact, origin, job
