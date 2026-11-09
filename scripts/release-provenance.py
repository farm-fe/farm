"""Pinned main-push release binding policy; no publication or signing API here.

Only the isolated reusable workflow may sign. Consumers invoke the reviewed
cryptographic CLI and never accept a caller-provided verification-result JSON.
"""
# cspell:words importlib sigstore
import importlib.util
import json
import os
from pathlib import Path
import re
import tempfile

ROOT=Path(__file__).resolve().parent

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,ROOT/file)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

policy=load('release_binding_policy','ci-attested-bindings.py')
envelope=load('release_binding_envelope','ci-provenance-consumer.py')
check=policy.check
REPOSITORY='farm-fe/farm'
REPOSITORY_ID=507542208
SIGNER='.github/workflows/release-provenance-signer.yaml'
NPM='.github/workflows/incremental-release.yaml'
CARGO='.github/workflows/cargo-source-release.yaml'
NATIVE_ABIS=['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc', 'linux-x64-musl', 'darwin-x64', 'win32-ia32-msvc', 'win32-arm64-msvc', 'linux-arm64-musl', 'linux-arm64-gnu']
CALLERS={'frozen':NPM,'public':NPM,'registry':NPM,'cargo':CARGO, **{'native-'+abi:NPM for abi in NATIVE_ABIS}}


def cells(scope):
    check(scope in CALLERS,'Unknown release signing scope')
    if scope.startswith('native-'):
        abi=scope.removeprefix('native-')
        group='User' if abi in NATIVE_ABIS[:3] else 'Release'
        return {'native':{'producer':'Build and Upload '+group+' Artifacts - '+abi,
            'uploadStep':'Upload formal native product catalog', 'artifactName':'formal-native-'+abi+'-{originAttempt}'}}
    if scope=='frozen':
        return {name:{'producer':'Build independent release candidates','uploadStep':step,
                      'artifactName':'farm-release-'+name+'-{sourceSHA}-{originAttempt}'}
                for name,step in [('seal','Upload independent release seal'),('candidates','Freeze independent release candidates')]}
    if scope=='registry':
        return {'public':{'producer':'Verify independent release public consumers',
            'uploadStep':'Upload independent release public acceptance',
            'artifactName':'farm-release-public-acceptance-{sourceSHA}-{originAttempt}'}}
    if scope=='public':
        receipts=load('release_signed_consumers','release-consumer-bindings.py')
        return {'public':{'producer':'Verify independent release public consumers','uploadStep':'Upload independent release public acceptance',
                          'artifactName':'farm-release-public-acceptance-{sourceSHA}-{originAttempt}'},
                **{kind+':'+platform:receipts.role(kind,platform)
                   for kind in ['candidate','public'] for platform in receipts.ABI}}
    return {'cargo':{'producer':'Prepare Cargo source archives','uploadStep':'Freeze verified Cargo source archives',
                     'artifactName':'cargo-source-{runId}'}}


def expected_policy(run, source, scope, signer_sha):
    check(scope in CALLERS and re.fullmatch('[a-f0-9]{40}',signer_sha or ''),'Reviewed release signer policy required')
    check(run.get('event')=='push' and run.get('head_branch')=='main' and run.get('path')==CALLERS[scope]
          and run.get('head_sha')==source['sha'] and run.get('repository',{}).get('id')==REPOSITORY_ID
          and run.get('head_repository',{}).get('id')==REPOSITORY_ID
          and run.get('repository',{}).get('full_name')==REPOSITORY,'Release signing requires original main push identity')
    check(type(run.get('id')) is int and run['id']>0 and type(run.get('run_attempt')) is int and 0<run['run_attempt']<=32,
          'Invalid release proof run identity')
    check(re.fullmatch('[a-f0-9]{40}',source.get('sha','')) and re.fullmatch('[a-f0-9]{40}',source.get('tree','')),
          'Exact release proof source required')
    return {'repository':REPOSITORY,'repositoryId':REPOSITORY_ID,'sourceSHA':source['sha'],'sourceTree':source['tree'],
            'signerWorkflow':SIGNER,'signerSHA':signer_sha,'callerWorkflow':CALLERS[scope],
            'runId':run['id'],'attempt':run['run_attempt'],'cells':cells(scope)}


def signer_identity(environment,scope):
    check(scope in CALLERS,'Unknown release signer scope')
    revision=environment.get('SIGNER_WORKFLOW_SHA','')
    check(re.fullmatch('[a-f0-9]{40}',revision) and environment.get('GITHUB_ACTIONS')=='true'
          and environment.get('RUNNER_ENVIRONMENT')=='github-hosted' and environment.get('GITHUB_JOB')=='sign'
          and environment.get('GITHUB_EVENT_NAME')=='push' and environment.get('GITHUB_REF')=='refs/heads/main'
          and environment.get('GITHUB_REPOSITORY')==REPOSITORY and environment.get('GITHUB_REPOSITORY_ID')==str(REPOSITORY_ID)
          and environment.get('GITHUB_WORKFLOW_REF')==REPOSITORY+'/'+CALLERS[scope]+'@refs/heads/main'
          and environment.get('SIGNER_WORKFLOW_REF')==REPOSITORY+'/'+SIGNER+'@'+revision,
          'Only isolated pinned release main push signer may prepare a proof')
    check(all(re.fullmatch('[1-9][0-9]*',environment.get(key,'')) for key in ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT']),
          'Release signer run identity missing')
    return revision


def prepare(transport,environment,scope,outputs,templates,*,prior_proof_provider=None):
    revision=signer_identity(environment,scope)
    restored=environment.get('SIGNER_CARGO_RESTORED')
    check(restored in ['true','false'] and (scope=='cargo' or restored=='false'),
          'Explicit Cargo restore mode required; other scopes cannot restore')
    run_id=int(environment['GITHUB_RUN_ID']);attempt=int(environment['GITHUB_RUN_ATTEMPT'])
    run=transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(attempt))
    check(run.get('id')==run_id and run.get('run_attempt')==attempt and run.get('head_sha')==environment.get('GITHUB_SHA'),
          'Release signer invocation differs from API')
    commit=transport.get('/git/commits/'+run['head_sha'])
    check(commit.get('sha')==run['head_sha'],'Release signer commit differs')
    expected=expected_policy(run,{'sha':run['head_sha'],'tree':commit['tree']['sha']},scope,revision)
    body=policy.build_manifest_from_fixed_outputs(transport,expected,outputs,templates)
    if scope=='cargo':
        row=json.loads(body)['bindings'][0]
        if restored=='true':
            check(row['originAttempt']<attempt,'Cargo restore must retain an earlier frozen origin')
            # Restore-script outputs are not upload-action outputs. Authenticate
            # their original signed binding independently in this pinned signer.
            def original(client,observed,source,kind):
                return verified_run(client,observed,source,kind,signer_sha=revision)
            provider=original if prior_proof_provider is None else prior_proof_provider
            metadata,origin,job=find_frozen_binding(transport,{'sha':expected['sourceSHA'],'tree':expected['sourceTree']},
                'cargo',run_id,attempt-1,row['artifactId'],'cargo',proof_provider=provider)
            check(metadata['digest']==row['artifactDigest'] and origin['run_attempt']==row['originAttempt']
                  and job['id']==row['jobId'],'Carried Cargo output differs from its original signed archive')
    return body


def verified_run(transport,run,source,scope,*,verify=None,signer_sha=None):
    # Optional function arguments are offline test seams, not CLI/env inputs.
    # Production supplies neither; it always loads the committed pin and CLI.
    if signer_sha is None:
        trust=policy.strict_json((ROOT/'release-provenance-trust.json').read_bytes())
        check(isinstance(trust,dict) and set(trust)=={'schemaVersion','signerSHA'} and trust['schemaVersion']==1,
              'Reviewed release signer trust configuration differs')
        signer_sha=trust['signerSHA']
    expected=expected_policy(run,source,scope,signer_sha)
    if verify is None:verify=load('release_pinned_verifier','ci-pinned-attestation-cli.py').verify_on_host
    # A frozen proof can outlive a failed later stage; the signed producer and
    # exact original attempt are checked independently. Latest status is captured
    # and rechecked, not replaced by success of an older overall workflow.
    route='/actions/runs/'+str(run['id'])
    latest=transport.get(route)
    check(all(latest.get(key)==run.get(key) for key in ['id','path','head_sha','event','head_branch','repository','head_repository'])
          and type(latest.get('run_attempt')) is int and run['run_attempt']<=latest['run_attempt']<=32,
          'Latest release run/source differs')
    if scope in ['public','registry'] or scope.startswith('native-'):
        check(latest['run_attempt']==run['run_attempt'],'Public acceptance or native cache requires the current authoritative attempt')
    name='farm-release-bindings-'+scope+'-'+str(run['id'])+'-'+str(run['run_attempt'])
    response=transport.get(route+'/artifacts?per_page=2&name='+name);rows=response.get('artifacts',[])
    check(response.get('total_count')==1 and len(rows)==1,'Release binding proof missing or ambiguous; original bytes must not be rebuilt')
    identity=rows[0].get('id');check(type(identity) is int and identity>0,'Invalid release proof artifact ID')
    artifact=transport.get('/actions/artifacts/'+str(identity))
    check(artifact.get('id')==identity and artifact.get('name')==name and artifact.get('expired') is False
          and artifact.get('workflow_run',{}).get('id')==run['id'] and type(artifact.get('size_in_bytes')) is int
          and 0<artifact['size_in_bytes']<=20*1024*1024,'Release proof envelope scope differs')
    files=envelope.proof_files(transport.archive(identity),artifact)
    with tempfile.TemporaryDirectory(prefix='farm-release-proof-') as directory:
        root=Path(directory)
        for filename,data in files.items():(root/filename).write_bytes(data)
        verified=verify(root/'ci-bindings.json',root/'ci-bindings.sigstore.json',expected)
    check(isinstance(verified,dict) and set(verified)==set(expected['cells']),'Verified release proof inventory differs')
    current=transport.get(route)
    check(all(current.get(key)==latest.get(key) for key in ['id','run_attempt','path','head_sha','event','head_branch','status','conclusion']),
          'Release state changed during verification')
    return expected,verified


def find_frozen_binding(transport, source, scope, run_id, maximum_attempt, artifact_id, cell, *, proof_provider=None):
    """Locate an original frozen identity, never rebuild or authenticate by name.

    Later attempts may repeat the same certified frozen artifact. The chosen
    proof must bind the requested immutable artifact ID and original producer.
    Missing or expired historical signatures are not repairable by repacking.
    """
    check(scope in ['frozen','cargo'] and cell in cells(scope), 'Unsupported frozen proof role')
    check(type(maximum_attempt) is int and 0<maximum_attempt<=32 and type(artifact_id) is int and artifact_id>0,
          'Invalid frozen proof selection')
    provider=verified_run if proof_provider is None else proof_provider
    for attempt in range(maximum_attempt,0,-1):
        run=transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(attempt))
        name='farm-release-bindings-'+scope+'-'+str(run_id)+'-'+str(attempt)
        result=transport.get('/actions/runs/'+str(run_id)+'/artifacts?per_page=2&name='+name)
        check(type(result.get('total_count')) is int and result['total_count']==len(result.get('artifacts',[]))
              and result['total_count']<=1,'Frozen proof discovery is ambiguous')
        if not result['artifacts']:continue
        expected,rows=provider(transport,run,source,scope)
        row=rows[cell]
        check(row['artifactId']==artifact_id,'Signed frozen artifact differs; cannot choose replacement bytes')
        meta,origin,job=policy.bound_artifact_metadata(transport,run,row,expected,maximum_bytes=2*1024**3)
        return meta,origin,job
    raise ValueError('Original frozen archive has no trusted signed producer; do not rebuild partial publication')


def signed_json(transport,run,source,scope,cell,filename='ledger.json',*,proof_provider=None):
    provider=verified_run if proof_provider is None else proof_provider
    expected,rows=provider(transport,run,source,scope)
    check(cell in rows,'Signed release receipt role missing')
    body,artifact,origin,job=policy.bound_artifact(transport,run,rows[cell],expected,
        required_conclusion=cells(scope)[cell].get("producerConclusion","success"))
    functional=load('release_signed_json','incremental-release-functional.py')
    return functional.exact_plan(body,artifact,filename),artifact,origin,job,expected,rows


def frozen_json(transport,run,source,cell,filename='ledger.json',*,proof_provider=None):
    check(cell=='seal','Only the frozen seal is a JSON receipt')
    name='farm-release-seal-'+source['sha']+'-'+str(run['run_attempt'])
    route='/actions/runs/'+str(run['id'])
    inventory=transport.get(route+'/artifacts?per_page=2&name='+name)
    rows=inventory.get('artifacts',[])
    check(inventory.get('total_count')==1 and len(rows)==1,'Original frozen seal missing or ambiguous')
    latest=transport.get(route)
    check(latest.get('id')==run['id'] and latest.get('head_sha')==source['sha'],'Frozen run source changed')
    artifact,origin,job=find_frozen_binding(transport,source,'frozen',run['id'],latest['run_attempt'],rows[0]['id'],cell,
                                          proof_provider=proof_provider)
    functional=load('release_frozen_json','incremental-release-functional.py')
    return functional.exact_plan(transport.archive(artifact['id']),artifact,filename),artifact,origin,job


def consumer_outputs(rows,kind):
    receipts=load('release_proven_consumer_roles','release-consumer-bindings.py')
    check(kind in ['candidate','public'],'Unknown signed consumer role')
    result={}
    for platform in receipts.ABI:
        row=rows[kind+':'+platform]
        result[platform]={'artifact_id':str(row['artifactId']),'artifact_digest':row['artifactDigest'].removeprefix('sha256:'),
                          'run_attempt':str(row['originAttempt'])}
    return result


def main():
    scope=os.environ.get('SIGNER_SCOPE','');signer_identity(os.environ,scope)
    directory=ROOT/'release-provenance-templates'/('native' if scope.startswith('native-') else scope)
    index=policy.strict_json((directory/'index.json').read_bytes())
    check(isinstance(index,list) and index and len(index)==len(set(index)),'Invalid reviewed release template index')
    templates={}
    for path in index:
        check(isinstance(path,str) and path.startswith('.github/workflows/') and '..' not in path.split('/'),'Invalid reviewed release template path')
        file=directory/path;check(file.is_file() and not file.is_symlink(),'Missing reviewed release template')
        templates[path]=file.read_bytes()
    api=load('release_signer_transport','ci-native-evidence.py')
    transport=api.GitHubReadOnly(REPOSITORY,os.environ.get('GH_TOKEN',''),allow_archive=True)
    body=prepare(transport,os.environ,scope,policy.strict_json(os.environ.get('SIGNER_UPLOAD_BINDINGS','').encode()),templates)
    output=Path('ci-provenance-output');output.mkdir()
    (output/'ci-bindings.json').write_bytes(body)
    (output/'predicate.json').write_text(json.dumps({'schemaVersion':1,'kind':'fixed-workflow-upload-bindings'}))


if __name__=='__main__':main()
