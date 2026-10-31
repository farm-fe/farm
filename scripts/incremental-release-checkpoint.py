"""Persistent repository Release receipt, not a signature or Actions attestation."""
# cspell:words importlib
import base64
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import re

spec=importlib.util.spec_from_file_location('release_checkpoint_evidence',Path(__file__).with_name('incremental-chain-evidence.py'))
evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)
chain=evidence.chain_plan
check=evidence.check
MARKER='farm-release-checkpoint-v1'

def create(manifest,source,baseline,proof,repository_id):
    check(proof.get('verified') is True and proof.get('manifestHash')==chain.digest(manifest)
          and proof.get('sourceSHA')==source['sha'] and proof.get('sourceTree')==source['tree'],'Verified public acceptance does not bind checkpoint source')
    check(manifest['profile']=='release' and manifest['chain']=='release' and manifest['namespace']=='farm-release-v1','CI cannot produce release checkpoints')
    check(manifest.get('baseline')==(baseline['transaction'] if baseline else None),'Checkpoint prior baseline differs from frozen transaction')
    prior=baseline['ledger']['packages'] if baseline else {}
    records={key:dict(row) for key,row in prior.items()}
    for row in manifest['packages']:
        identity=row['kind']+':'+row['name']
        records[identity]={**row,'identity':identity,'originSourceSHA':source['sha'],'originRunId':manifest['producerRunId'],'originAttempt':manifest['producerAttempt']}
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Checkpoint source graph is ambiguous')
    expected=chain.independent_family(source['files'],nodes)
    check(set(records)==set(expected) and all(row['version']==expected[key]['version'] for key,row in records.items()),'Checkpoint complete family coverage differs')
    return {'schemaVersion':1,'kind':'repository-release-checkpoint-not-actions-attestation','chain':'release','profile':'release',
            'namespace':'farm-release-v1','repositoryId':repository_id,'manifestHash':chain.digest(manifest),
            'sourceSHA':source['sha'],'sourceTree':source['tree'],'planHash':manifest['planHash'],
            'sourceSnapshotHash':chain.digest(chain.snapshot_records(source['files'],source['modes'])),
            'producerRunId':manifest['producerRunId'],'producerAttempt':manifest['producerAttempt'],
            'publicAcceptanceHash':chain.digest(proof),'packages':records}

def encode(value):
    body=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()
    check(len(body)<=4*1024**2,'Release checkpoint expands beyond bound')
    encoded=base64.b64encode(gzip.compress(body,mtime=0)).decode()
    check(len(encoded)<=60000,'Persistent release checkpoint exceeds body quota')
    return '<!-- '+MARKER+':'+hashlib.sha256(body).hexdigest()+':'+encoded+' -->'

def decode(release,manifest,source,repository_id):
    check(release.get('draft') is False and release.get('prerelease') is False
          and release.get('author',{}).get('login')=='github-actions[bot]' and release.get('author',{}).get('type')=='Bot','Persistent receipt is not the repository Actions publisher Release')
    matches=re.findall(r'<!-- '+MARKER+r':([a-f0-9]{64}):([A-Za-z0-9+/=]+) -->',release.get('body',''))
    check(len(matches)==1 and len(matches[0][1])<=60000,'Persistent release checkpoint missing or ambiguous')
    packed=base64.b64decode(matches[0][1],validate=True)
    import io
    with gzip.GzipFile(fileobj=io.BytesIO(packed)) as stream:body=stream.read(4*1024**2+1)
    check(len(body)<=4*1024**2 and hashlib.sha256(body).hexdigest()==matches[0][0],'Persistent checkpoint bytes differ')
    value=json.loads(body.decode('utf-8'))
    check(body==json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode(),'Persistent checkpoint JSON is not canonical')
    expected={'schemaVersion':1,'kind':'repository-release-checkpoint-not-actions-attestation','chain':'release','profile':'release','namespace':'farm-release-v1',
              'repositoryId':repository_id,'manifestHash':chain.digest(manifest),'sourceSHA':source['sha'],'sourceTree':source['tree'],
              'planHash':manifest['planHash'],'sourceSnapshotHash':chain.digest(chain.snapshot_records(source['files'],source['modes'])),
              'producerRunId':manifest['producerRunId'],'producerAttempt':manifest['producerAttempt']}
    check(all(value.get(key)==wanted for key,wanted in expected.items()),'Persistent checkpoint source, chain or producer differs')
    check('<!-- farm-release-manifest:'+expected['manifestHash']+' -->' in release['body'],'Release does not bind checkpoint manifest')
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Persistent checkpoint source graph is ambiguous')
    family=chain.independent_family(source['files'],nodes);records=value.get('packages',{})
    check(set(records)==set(family) and all(row.get('version')==family[key]['version'] and row.get('identity')==key
          and re.fullmatch('[a-f0-9]{64}',row.get('archiveSha256','')) for key,row in records.items()),'Persistent checkpoint family differs')
    chain.exact(value.get('publicAcceptanceHash'),64)
    for row in records.values():
        chain.exact(row.get('originSourceSHA'),40)
        check(type(row.get('originRunId')) is int and row['originRunId']>0 and type(row.get('originAttempt')) is int and row['originAttempt']>0,'Persistent original producer identity missing')
    return value
