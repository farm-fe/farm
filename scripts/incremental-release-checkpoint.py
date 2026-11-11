"""Persistent repository Release receipt, not a signature or Actions attestation."""
# cspell:words importlib finalizer
import base64
import copy
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
PRESENCE_POLICY='registry-exact-version-v2'
DISPOSITION_FIELDS=('originKind','name','version','integrity','metadata','tarballURL',
                    'latestVerified','uploadAttempted','evidenceKind','registryBytesVerified')

def load_registry_selection():
    spec=importlib.util.spec_from_file_location('checkpoint_registry_selection',Path(__file__).with_name('release-registry-selection.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def presence_records(manifest,source,proof):
    """Bind each origin to its verified evidence, never discarded candidates."""
    check(isinstance(proof,dict) and proof.get('verified') is True
          and proof.get('kind')=='release-version-presence-v1'
          and proof.get('manifestHash')==chain.digest(manifest)
          and proof.get('sourceSHA')==source['sha'] and proof.get('sourceTree')==source['tree'],
          'Version-presence checkpoint requires verified public proof')
    check(manifest.get('sourceSHA')==source['sha'] and manifest.get('sourceTree')==source['tree']
          and manifest.get('publicationScope')=='npm' and manifest.get('baseline') is None
          and manifest.get('releaseValidation')=='archive-integrity-v1'
          and manifest.get('buildFailures',{})=={}, 'Version-presence checkpoint source or policy differs')
    snapshot=manifest.get('registrySnapshot')
    check(isinstance(snapshot,dict) and snapshot.get('schemaVersion')==2,
          'Version-presence checkpoint observation policy differs')
    selector=load_registry_selection()
    family=selector.validate(source,'npm',snapshot)
    dispositions=proof.get('dispositions')
    check(isinstance(dispositions,dict) and set(dispositions)==set(family),
          'Version-presence checkpoint public family differs')
    sealed={}
    for row in manifest['packages']:
        check(isinstance(row,dict) and row.get('kind')=='npm' and isinstance(row.get('name'),str),
              'Version-presence checkpoint sealed identity invalid')
        identity='npm:'+row['name']
        check(identity in family and identity not in sealed and row.get('version')==family[identity]['version'],
              'Version-presence checkpoint sealed family differs')
        sealed[identity]=row
    missing={key for key,row in snapshot['packages'].items() if row['status']=='missing'}
    check(set(sealed)==missing and all(row['status'] in ['missing','published']
          for row in snapshot['packages'].values()), 'Version-presence checkpoint has incomplete build or registry evidence')
    check(type(manifest.get('producerRunId')) is int and manifest['producerRunId']>0
          and type(manifest.get('producerAttempt')) is int and manifest['producerAttempt']>0,
          'Version-presence checkpoint producer identity missing')
    records={}
    for identity,family_row in family.items():
        actual=dispositions[identity]
        check(isinstance(actual,dict) and all(field in actual for field in DISPOSITION_FIELDS),
              'Version-presence checkpoint disposition fields missing')
        selector.validate_observed(family_row,{**actual,'status':'published'},2)
        check(actual['latestVerified'] is True and type(actual['uploadAttempted']) is bool,
              'Version-presence checkpoint latest or upload proof differs')
        observed=snapshot['packages'][identity]
        origin=actual['originKind']
        public={field:actual[field] for field in DISPOSITION_FIELDS}
        if origin=='source':
            check(actual['evidenceKind']=='registry-archive' and actual['registryBytesVerified'] is True
                  and isinstance(actual.get('archiveSha256'),str) and re.fullmatch('[a-f0-9]{64}',actual['archiveSha256'])
                  and type(actual.get('archiveBytes')) is int and 0<actual['archiveBytes']<=100*1024**2,
                  'Source checkpoint requires verified public archive bytes')
            public.update(archiveSha256=actual['archiveSha256'],archiveBytes=actual['archiveBytes'])
        else:
            check(actual['evidenceKind']=='registry-metadata' and actual['registryBytesVerified'] is False
                  and 'archiveSha256' not in actual and 'archiveBytes' not in actual,
                  'Registry-origin checkpoint cannot invent archive verification')
        if observed['status']=='published':
            check(origin=='registry' and actual['uploadAttempted'] is False,
                  'Retained checkpoint cannot claim source or race origin')
            check(all(actual[field]==observed[field] for field in
                      ['name','version','integrity','metadata']),
                  'Retained checkpoint differs from verified registry observation')
            record={**family_row,**public,'identity':identity}
        else:
            check(origin in ['source','registry-race'], 'Missing-version checkpoint origin differs')
            if origin=='source':
                row=sealed[identity]
                check(actual['uploadAttempted'] is True
                      and actual['archiveSha256']==row.get('archiveSha256')
                      and actual['archiveBytes']==row.get('archiveBytes'),
                      'Source checkpoint differs from sealed archive or upload proof')
                record={**family_row,**row,**public,'identity':identity,
                        'originSourceSHA':source['sha'],'originSourceTree':source['tree'],
                        'originRunId':manifest['producerRunId'],'originAttempt':manifest['producerAttempt']}
                check(all(record.get(field)==value for field,value in row.items()),
                      'Source checkpoint changes a sealed manifest field')
            else:
                check(actual['uploadAttempted'] is False, 'Registry-race checkpoint cannot claim a source upload')
                # The exact version appeared before any upload attempt. Only
                # metadata was read; no candidate archive hash or source
                # producer fields may become registry verification claims.
                record={**family_row,**public,'identity':identity}
        records[identity]=copy.deepcopy(record)
    if 'packages' in proof:
        check(proof['packages']=={identity:row.get('archiveSha256') for identity,row in records.items()},
              'Version-presence checkpoint public checksum coverage differs')
    return records

def create(manifest,source,baseline,proof,repository_id):
    check(proof.get('verified') is True and proof.get('manifestHash')==chain.digest(manifest)
          and proof.get('sourceSHA')==source['sha'] and proof.get('sourceTree')==source['tree'],'Verified public acceptance does not bind checkpoint source')
    check(manifest['profile']=='release' and manifest['chain']=='release' and manifest['namespace']=='farm-release-v1','CI cannot produce release checkpoints')
    expected_baseline=manifest['registrySnapshot'].get('stateBaseline') if manifest.get('registrySnapshot') is not None else (baseline['transaction'] if baseline else None)
    check(manifest.get('baseline')==expected_baseline,'Checkpoint prior baseline differs from frozen transaction')
    presence=manifest.get('selectionPolicy')==PRESENCE_POLICY
    if presence:
        check(baseline is None, 'Version-presence checkpoint cannot inherit historical rows')
        records=presence_records(manifest,source,proof)
    else:
        prior=baseline['ledger']['packages'] if baseline else {}
        records={key:dict(row) for key,row in prior.items()}
        if manifest.get('registrySnapshot') is not None:
            selector=load_registry_selection()
            family=selector.validate(source,'npm',manifest['registrySnapshot'])
            for key,row in manifest['registrySnapshot']['packages'].items():
                if row['status']=='published':
                    records[key]={**family[key],**row,'identity':key,'originKind':'independently-verified-registry'}
        for row in manifest['packages']:
            identity=row['kind']+':'+row['name']
            records[identity]={**row,'identity':identity,'originSourceSHA':source['sha'],'originRunId':manifest['producerRunId'],'originAttempt':manifest['producerAttempt']}
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Checkpoint source graph is ambiguous')
    expected=chain.independent_family(source['files'],nodes)
    if manifest.get('publicationScope') == 'npm': expected={key:row for key,row in expected.items() if row['kind']=='npm'}
    check(set(records)==set(expected) and all(row['version']==expected[key]['version'] for key,row in records.items()),'Checkpoint complete family coverage differs')
    return {'schemaVersion':1,'kind':'repository-release-checkpoint-not-actions-attestation','chain':'release','profile':'release',
            'namespace':'farm-release-v1','publicationScope':manifest.get('publicationScope','all'),'repositoryId':repository_id,'manifestHash':chain.digest(manifest),
            'sourceSHA':source['sha'],'sourceTree':source['tree'],'planHash':manifest['planHash'],
            'sourceSnapshotHash':chain.digest(chain.snapshot_records(source['files'],source['modes'])),
            'producerRunId':manifest['producerRunId'],'producerAttempt':manifest['producerAttempt'],
            'publicAcceptanceHash':chain.digest(proof),'packages':records,
            **({'selectionPolicy':PRESENCE_POLICY} if presence else {})}

def encode(value):
    body=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()
    check(len(body)<=4*1024**2,'Release checkpoint expands beyond bound')
    encoded=base64.b64encode(gzip.compress(body,mtime=0)).decode()
    check(len(encoded)<=60000,'Persistent release checkpoint exceeds body quota')
    return '<!-- '+MARKER+':'+hashlib.sha256(body).hexdigest()+':'+encoded+' -->'

def decode(release,manifest,source,repository_id,proof=None):
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
    if manifest.get('selectionPolicy')==PRESENCE_POLICY:
        check(isinstance(proof,dict), 'Version-presence checkpoint requires verified public proof')
        wanted=create(manifest,source,None,proof,repository_id)
        # The body checksum only detects transport corruption. Every v2 row,
        # origin, source producer and public-acceptance hash must independently
        # match the verified public proof supplied by the finalizer.
        check(value==wanted, 'Persistent version-presence checkpoint differs from verified public proof')
        return value
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Persistent checkpoint source graph is ambiguous')
    family=chain.independent_family(source['files'],nodes);records=value.get('packages',{})
    check(value.get('publicationScope','all') == manifest.get('publicationScope','all'),'Checkpoint publication scope differs')
    if manifest.get('publicationScope') == 'npm': family={key:row for key,row in family.items() if row['kind']=='npm'}
    check(set(records)==set(family) and all(row.get('version')==family[key]['version'] and row.get('identity')==key
          and re.fullmatch('[a-f0-9]{64}',row.get('archiveSha256','')) for key,row in records.items()),'Persistent checkpoint family differs')
    chain.exact(value.get('publicAcceptanceHash'),64)
    for row in manifest['packages']:
        identity = row['kind'] + ':' + row['name']
        expected_record = {**row, 'identity': identity, 'originSourceSHA': source['sha'],
                           'originRunId': manifest['producerRunId'], 'originAttempt': manifest['producerAttempt']}
        check(records.get(identity) == expected_record, 'Published checkpoint row differs from frozen archive or producer')
    for identity,row in records.items():
        if row.get('originKind')=='independently-verified-registry':
            snapshot=manifest.get('registrySnapshot',{})
            observed=snapshot.get('packages',{}).get(identity,{})
            check(observed.get('status')=='published' and all(row.get(key)==observed.get(key) for key in ['name','version','archiveSha256','integrity','metadata']),'Registry retained checkpoint differs from frozen observation')
            continue
        chain.exact(row.get('originSourceSHA'),40)
        check(type(row.get('originRunId')) is int and row['originRunId']>0 and type(row.get('originAttempt')) is int and row['originAttempt']>0,'Persistent original producer identity missing')
    return value
