"""Fixed non-matrix needs bindings for release consumer receipts.

Bindings must come from reviewed workflow needs outputs, or a separately
cryptographically verified release manifest. Never derive them from receipts.
"""
# cspell:words importlib
import importlib.util
import json
import re
from pathlib import Path


def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

execution=load('release_receipt_execution','ci-execution-evidence.py')
functional=load('release_receipt_archive','incremental-release-functional.py')
check=execution.check
PLATFORMS={'linux':'ubuntu-24.04','macos':'macos-latest','windows':'windows-latest'}
FIELDS={'artifact_id':r'[1-9][0-9]*','artifact_digest':r'[a-f0-9]{64}','run_attempt':r'[1-9][0-9]*'}
ABI={'ubuntu-24.04':{'os':'linux','arch':'x64','abi':'linux-x64-gnu'},
     'macos-latest':{'os':'darwin','arch':'arm64','abi':'darwin-arm64'},
     'windows-latest':{'os':'win32','arch':'x64','abi':'win32-x64-msvc'}}


def outputs(value):
    # Strict JSON rejects duplicate keys before the exact interface check.
    policy=load('release_receipt_json','ci-attested-bindings.py')
    row=policy.strict_json(value.encode()) if isinstance(value,str) else value
    check(isinstance(row,dict) and set(row)=={label+'_'+field for label in PLATFORMS for field in FIELDS},
          'Missing or unexpected fixed release consumer outputs')
    result={}
    for label,platform in PLATFORMS.items():
        binding={field:row[label+'_'+field] for field in FIELDS}
        check(all(isinstance(binding[field],str) and re.fullmatch(pattern,binding[field]) for field,pattern in FIELDS.items()),
              'Malformed fixed release consumer outputs')
        result[platform]=binding
    check(len({row['artifact_id'] for row in result.values()})==3,'Release platform artifacts must be distinct')
    return result


def role(kind,platform):
    check(kind in ['candidate','public'] and platform in ABI,'Unknown release consumer role')
    if kind=='candidate':
        return {'producer':'Independent release candidate consumer ('+platform+')',
                'uploadStep':'Upload independent release candidate acceptance',
                'artifactName':'farm-release-candidate-consumer-{sourceSHA}-'+platform+'-{originAttempt}'}
    return {'producer':'Independent public registry consumer ('+platform+')',
            'uploadStep':'Upload independent public consumer report',
            'artifactName':'release-public-'+platform+'-{originAttempt}'}


def receipt(transport,run,manifest,kind,platform,trusted_output):
    check(run.get('id')==manifest['producerRunId'] and run.get('head_sha')==manifest['sourceSHA']
          and run.get('path')=='.github/workflows/incremental-release.yaml' and run.get('head_branch')=='main'
          and run.get('event')=='push' and run.get('repository',{}).get('id')==507542208
          and run.get('head_repository',{}).get('id')==507542208,'Release consumer run or source differs')
    definition=role(kind,platform)
    origin,job=execution.explicit_producer(transport,run,definition['producer'],trusted_output)
    attempt=origin['run_attempt']
    check(attempt>=manifest['producerAttempt'],'Consumer predates frozen producer')
    artifact=transport.get('/actions/artifacts/'+trusted_output['artifact_id'])
    name=definition['artifactName'].format(sourceSHA=manifest['sourceSHA'],originAttempt=attempt)
    check(str(artifact.get('id'))==trusted_output['artifact_id'] and artifact.get('name')==name
          and artifact.get('workflow_run',{}).get('id')==run['id'] and artifact.get('expired') is False,
          'Release consumer artifact or platform differs')
    execution.upload_binding(job,definition['uploadStep'],artifact,trusted_output)
    report=functional.exact_plan(transport.archive(artifact['id']),artifact,'result.json')
    import hashlib
    manifest_hash=hashlib.sha256(json.dumps(manifest,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    binding={'manifestHash':manifest_hash,'sourceSHA':manifest['sourceSHA'],'sourceTree':manifest['sourceTree'],
             'producerRunId':manifest['producerRunId'],'producerAttempt':manifest['producerAttempt'],
             'consumerRunId':run['id'],'consumerAttempt':attempt}
    if kind=='candidate':
        check(report.get('releaseBinding')==binding and report.get('status')=='passed'
              and report.get('scratchRemoved') is True and report.get('cleanupConfirmed') is not False,
              'Candidate receipt frozen bytes or cleanup differ')
        check(report.get('platform')==ABI[platform],'Candidate receipt ABI differs')
    else:
        check(report.get('publicationBinding')=={**binding,'platform':platform}
              and report.get('status')=='passed' and report.get('publicRegistry') is True
              and report.get('workspacePatches') is False and report.get('cleanupConfirmed') is True,
              'Public receipt frozen bytes or cleanup differ')
        check({'npm-install','esm-cjs','types','docs-build','sdk'}.issubset(report.get('stages',[])),
              'Public consumer stages missing')
    return report,artifact,origin,job
