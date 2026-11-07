"""Discover an untrusted proof envelope, then authenticate its exact run policy.

Archive names locate proof bytes only. The verifier must be the pinned CLI
adapter; tests replace that boundary explicitly and do not claim cryptography.
"""
# cspell:words importlib sigstore
import hashlib
import importlib.util
import io
from pathlib import Path
import re
import stat
import tempfile
import zipfile


def load_module(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

signer=load_module('consumer_signer_policy','ci-provenance-signer.py')
bindings=signer.bindings
check=bindings.check


def proof_files(body, artifact):
    maximum=20*1024*1024
    check(isinstance(body,bytes) and 0<len(body)<=maximum and len(body)==artifact.get('size_in_bytes')
          and artifact.get('digest')=='sha256:'+hashlib.sha256(body).hexdigest(),'Binding proof archive digest or size differs')
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        rows=archive.infolist()
        names={'ci-bindings.json','ci-bindings.sigstore.json'}
        check(len(rows)==2 and {row.filename for row in rows}==names
              and all(not row.is_dir() and stat.S_IFMT(row.external_attr>>16) in [0,stat.S_IFREG] for row in rows),
              'Binding proof archive members differ')
        for row in rows:
            limit=4*1024*1024 if row.filename=='ci-bindings.json' else 16*1024*1024
            check(0<row.file_size<=limit,'Binding proof expanded size differs')
        return {name:archive.read(name) for name in names}


def load_verified_run(transport, run, source, signer_sha, *, verify):
    """verify is a trusted process adapter, never an artifact-supplied result.

    The latest run must still be the exact completed attempt. A proof from a
    successful old attempt is not accepted for a newer failure OR success.
    """
    check(re.fullmatch(r'[a-f0-9]{40}',signer_sha or ''),'Reviewed signer revision required')
    scope=next((key for key,path in signer.CALLERS.items() if run.get('path')==path),None)
    check(scope is not None and run.get('event')=='push' and run.get('head_branch')=='main'
          and run.get('head_sha')==source['sha'] and run.get('repository',{}).get('id')==signer.REPOSITORY_ID
          and run.get('head_repository',{}).get('id')==signer.REPOSITORY_ID
          and run.get('repository',{}).get('full_name')==signer.REPOSITORY,
          'Only exact main CI/Unit evidence can supply signed bindings')
    check(type(run.get('id')) is int and run['id']>0 and type(run.get('run_attempt')) is int and 0<run['run_attempt']<=32,
          'Invalid signed run identity')
    check(run.get('status')=='completed' and run.get('conclusion')=='success','Signed run did not succeed')
    def latest_unchanged():
        latest=transport.get('/actions/runs/'+str(run['id']))
        check(all(latest.get(key)==run.get(key) for key in ['id','run_attempt','path','head_sha','event','head_branch','status','conclusion'])
              and latest.get('repository',{}).get('id')==signer.REPOSITORY_ID
              and latest.get('head_repository',{}).get('id')==signer.REPOSITORY_ID,
              'Latest signed run changed; old proof cannot authorize a new attempt')
    latest_unchanged()
    expected={'repository':signer.REPOSITORY,'repositoryId':signer.REPOSITORY_ID,'sourceSHA':source['sha'],'sourceTree':source['tree'],
              'signerWorkflow':signer.SIGNER,'signerSHA':signer_sha,'callerWorkflow':run['path'],
              'runId':run['id'],'attempt':run['run_attempt'],'cells':signer.cells(scope)}
    name='farm-ci-bindings-'+scope+'-'+str(run['id'])+'-'+str(run['run_attempt'])
    result=transport.get('/actions/runs/'+str(run['id'])+'/artifacts?per_page=2&name='+name)
    rows=result.get('artifacts',[])
    check(result.get('total_count')==1 and len(rows)==1,'Signed binding proof missing or ambiguous')
    check(type(rows[0].get('id')) is int and rows[0]['id']>0,'Invalid proof artifact ID')
    artifact=transport.get('/actions/artifacts/'+str(rows[0]['id']))
    check(artifact.get('id')==rows[0]['id'] and artifact.get('name')==name and artifact.get('expired') is False
          and artifact.get('workflow_run',{}).get('id')==run['id'],'Binding proof artifact scope differs')
    check(type(artifact.get('size_in_bytes')) is int and 0<artifact['size_in_bytes']<=20*1024*1024,'Proof archive size exceeded before download')
    files=proof_files(transport.archive(artifact['id']),artifact)
    with tempfile.TemporaryDirectory(prefix='farm-verified-bindings-') as directory:
        root=Path(directory)
        for filename,body in files.items():(root/filename).write_bytes(body)
        # The adapter returns ONLY policy-validated rows from successful crypto.
        verified=verify(root/'ci-bindings.json',root/'ci-bindings.sigstore.json',expected)
    check(isinstance(verified,dict) and set(verified)==set(expected['cells']),'Verified binding inventory differs')
    latest_unchanged()
    return expected,verified
