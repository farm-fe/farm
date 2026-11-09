"""Read a completed release baseline; Git state alone never proves publication."""
# cspell:words importlib urllib
import base64
import hashlib
import importlib.util
import json
from pathlib import Path
import re

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

evidence=load('published_baseline_evidence','incremental-chain-evidence.py')
execution=load('published_baseline_execution','ci-execution-evidence.py')
chain=evidence.chain_plan
check=evidence.check
checkpoint=load('published_release_checkpoint','incremental-release-checkpoint.py')
provenance=load('published_release_provenance','release-provenance.py')
consumers=load('published_release_consumers','release-consumer-bindings.py')
WORKFLOW='.github/workflows/incremental-release.yaml'

class ReleaseBaselineReader:
    def __init__(self,transport,registry,read_source=evidence.source_reader,*,proof_provider=None):
        self.transport,self.registry,self.read_source=transport,registry,read_source
        self.proof_provider=proof_provider
    def state(self):
        refs=self.transport.get('/git/matching-refs/heads/farm-release-state')
        check(isinstance(refs,list) and len(refs)<=8,'Publication state ref inventory exceeds bound')
        rows=[row for row in refs if row['ref']=='refs/heads/farm-release-state']
        check(len(rows)<=1,'Ambiguous publication state ref')
        if not rows:return None,{'baseline':None,'active':None,'completed':{}}
        obj=rows[0]['object'];check(obj['type']=='commit','Publication state ref is not a commit')
        sha=chain.exact(obj['sha'],40)
        commit=self.transport.get('/git/commits/'+sha)
        tree=self.transport.get('/git/trees/'+chain.exact(commit['tree']['sha'],40))
        check(tree.get('truncated') is False and len(tree['tree'])==1,'Publication state tree differs')
        row=tree['tree'][0]
        check(row['path']=='publication-state.json' and row['mode']=='100644' and row['type']=='blob','Publication state is not one regular JSON member')
        blob=self.transport.get('/git/blobs/'+chain.exact(row['sha'],40))
        check(blob.get('encoding')=='base64' and type(blob.get('size')) is int and 0<blob['size']<=4*1024**2,'Publication state blob exceeds bound')
        body=base64.b64decode(''.join(blob['content'].splitlines()),validate=True)
        check(len(body)==blob['size'] and hashlib.sha1(b'blob '+str(len(body)).encode()+b'\0'+body).hexdigest()==row['sha'],'Publication state Git blob bytes differ')
        value=json.loads(body.decode('utf-8'));check(isinstance(value.get('completed'),dict),'Publication completion inventory missing')
        return sha,value
    def persistent(self,manifest,repository):
        tag=manifest.get('tag');check(isinstance(tag,str) and re.fullmatch(r'(?:v\d+\.\d+\.\d+|packages-[a-f0-9]{16})',tag),'Persistent baseline tag missing')
        release=self.transport.get('/releases/tags/'+tag)
        if checkpoint.MARKER not in release.get('body',''):return None
        source=self.read_source(chain.exact(manifest.get('sourceSHA'),40))
        main=chain.exact(self.transport.get('/branches/main')['commit']['sha'],40)
        if main!=source['sha']:
            comparison=self.transport.get('/compare/'+source['sha']+'...'+main)
            check(comparison.get('status') in ['ahead','identical'] and comparison.get('base_commit',{}).get('sha')==source['sha']
                  and comparison.get('merge_base_commit',{}).get('sha')==source['sha'],'Persistent checkpoint source is not trusted main history')
        obj=self.transport.get('/git/ref/tags/'+tag)['object']
        for _ in range(5):
            if obj['type']=='tag':obj=self.transport.get('/git/tags/'+chain.exact(obj['sha'],40))['object']
        check(obj.get('type')=='commit' and obj.get('sha')==source['sha'],'Persistent checkpoint tag source differs')
        value=checkpoint.decode(release,manifest,source,repository['id'])
        return value,source
    def candidates(self,manifest,repository,last_attempt,*,trusted_outputs=None):
        run_id,first=manifest['producerRunId'],manifest['producerAttempt']
        check(type(last_attempt) is int and first<=last_attempt<=32,'Candidate acceptance history exceeds bound')
        run=self.transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(last_attempt))
        check(run.get('repository',{}).get('id')==repository['id'],'Candidate repository differs')
        if trusted_outputs is None:
            source={'sha':manifest['sourceSHA'],'tree':manifest['sourceTree']}
            provider=provenance.verified_run if self.proof_provider is None else self.proof_provider
            _,rows=provider(self.transport,run,source,'public')
            trusted_outputs=provenance.consumer_outputs(rows,'candidate')
        check(isinstance(trusted_outputs,dict) and set(trusted_outputs)==set(consumers.ABI),
              'Exact trusted candidate platform inventory required')
        result={}
        for platform,output in trusted_outputs.items():
            _,artifact,origin,_=consumers.receipt(self.transport,run,manifest,'candidate',platform,output)
            result[platform]={'attempt':origin['run_attempt'],'artifactId':artifact['id'],'artifactDigest':artifact['digest']}
        return result
    def completed(self,transaction,record,repository):
        manifest=record.get('manifest',{})
        check(chain.digest(manifest)==transaction,'Completed manifest digest differs')
        check(manifest.get('chain')=='release' and manifest.get('profile')=='release' and manifest.get('namespace')=='farm-release-v1','Cross-chain completed manifest rejected')
        source=self.read_source(chain.exact(manifest.get('sourceSHA'),40))
        check(source['tree']==manifest.get('sourceTree'),'Completed source tree differs')
        run_id,attempt=manifest.get('producerRunId'),manifest.get('producerAttempt')
        check(type(run_id) is int and run_id>0 and type(attempt) is int and attempt>0,'Completed producer identity missing')
        run=self.transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(attempt))
        check(run['id']==run_id and run['run_attempt']==attempt and run['head_sha']==source['sha'] and run['path']==WORKFLOW
              and run['status']=='completed'
              and run['event'] in ['push','workflow_dispatch'] and run['head_branch']=='main'
              and run['repository']['id']==repository['id']==run['head_repository']['id'],'Completed producer provenance differs')
        seal,artifact,origin,producer=provenance.frozen_json(self.transport,run,source,'seal',proof_provider=self.proof_provider)
        check(origin['run_attempt']==attempt,'Completed frozen origin differs')
        check(seal.get('manifestHash')==transaction and seal.get('profile')=='release' and seal.get('namespace')=='farm-release-v1'
              and seal.get('sourceTree')==source['tree'] and seal.get('producerJobId')==producer['id']
              and seal.get('packageInputs')==evidence.package_input_hashes(source),'Completed seal or source inputs differ')
        latest=self.transport.get('/actions/runs/'+str(run_id))
        check(type(latest['run_attempt']) is int and attempt<=latest['run_attempt']<=32,'Latest release attempt differs')
        entry=load('baseline_release_entry','incremental-release-entry.py')
        public,_,_,job,_,signed=provenance.signed_json(self.transport,latest,source,entry.public_scope(manifest),'public',proof_provider=self.proof_provider)
        if entry.archive_integrity_mode(manifest):
            check(public.get('kind')=='release-byte-integrity-v1', 'Registry proof kind differs')
        else:
            self.candidates(manifest,repository,latest['run_attempt'],trusted_outputs=provenance.consumer_outputs(signed,'candidate'))
        check(public.get('manifestHash')==transaction and public.get('sourceTree')==source['tree']
              and public.get('planHash')==manifest['planHash'] and public.get('producerJobId')==job['id'],
              'Completed public acceptance identity differs')
        expected={row['kind']+':'+row['name']:row['archiveSha256'] for row in manifest['packages']}
        check(public.get('packages')==expected and load('baseline_release_entry','incremental-release-entry.py').public_integrity_verified(public, manifest),
              'Completed public consumer coverage differs')
        publisher=load('completed_public_consumer_validation','incremental-release-publish.py')
        if not entry.archive_integrity_mode(manifest):
            publisher.verify_public_selection(self.transport,manifest,run_id,latest['run_attempt'],public.get('consumerReports'),
                                             upload_outputs=provenance.consumer_outputs(signed,'public'))
        tag=manifest.get('tag');check(isinstance(tag,str) and re.fullmatch(r'(?:v\d+\.\d+\.\d+|packages-[a-f0-9]{16})',tag),'Completed tag identity missing')
        obj=self.transport.get('/git/ref/tags/'+tag)['object']
        for _ in range(5):
            if obj['type']=='tag':obj=self.transport.get('/git/tags/'+chain.exact(obj['sha'],40))['object']
        check(obj.get('type')=='commit' and obj.get('sha')==source['sha'],'Completed tag does not point to source')
        release=self.transport.get('/releases/tags/'+tag)
        check(release.get('draft') is False and release.get('prerelease') is False
              and '<!-- farm-release-manifest:'+transaction+' -->' in release.get('body',''),'Completed GitHub Release read-back differs')
        return manifest,source
    def lookup(self,transaction=None):
        revision,state=self.state()
        selected=state.get('baseline') if transaction is None else transaction
        if selected is None:return None
        repository=self.transport.get('');check(repository['default_branch']=='main','Release baseline default branch differs')
        records={};visited=set();cursor=selected;latest_source=None;latest_manifest=None
        check(selected in state['completed'],'Completed baseline record missing')
        latest_manifest=state['completed'][selected].get('manifest',{})
        check(chain.digest(latest_manifest)==selected,'Persistent baseline transaction differs')
        durable=self.persistent(latest_manifest,repository)
        if durable is not None:
            value,latest_source=durable;records=value['packages'];cursor=None
        while cursor is not None:
            check(cursor not in visited and len(visited)<32,'Completed release lineage is cyclic or exceeds bound');visited.add(cursor)
            check(cursor in state['completed'],'Completed release lineage missing')
            manifest,source=self.completed(cursor,state['completed'][cursor],repository)
            if latest_source is None:latest_source,latest_manifest=source,manifest
            for row in manifest['packages']:
                identity=row['kind']+':'+row['name']
                if identity not in records:records[identity]={**row,'identity':identity,'originRunId':manifest['producerRunId'],'originAttempt':manifest['producerAttempt'],'originSourceSHA':source['sha']}
            cursor=manifest.get('baseline')
        nodes,issues=chain.repository_graph(latest_source['files']);check(not issues,'Published baseline graph is ambiguous')
        expected=chain.independent_family(latest_source['files'],nodes)
        if latest_manifest.get('publicationScope') == 'npm': expected={key:row for key,row in expected.items() if row['kind']=='npm'}
        check(set(records)==set(expected) and all(row['version']==expected[key]['version'] for key,row in records.items()),'Completed baseline family or version coverage differs')
        checksums=self.registry.verify(records)
        check(checksums=={key:row['archiveSha256'] for key,row in records.items()},'Completed registry byte checksums differ')
        observed=getattr(self.registry,'verifiedArchives',{})
        for key,row in records.items():
            if isinstance(observed,dict) and key in observed:
                row['tarball']=observed[key]['tarball']
                if row['kind']=='npm':row['integrity']='sha512-'+observed[key]['sha512']
            for field in ['manifestPath','owner','platform']:
                if field in expected[key]:row[field]=expected[key][field]
        ledger={'schemaVersion':1,'publicationScope':latest_manifest.get('publicationScope','all'),'chain':'release','profile':'release','policy':chain.POLICY,'status':'success','registryVerified':True,
                'repositoryId':repository['id'],'runId':latest_manifest['producerRunId'],'runAttempt':latest_manifest['producerAttempt'],
                'sourceSHA':latest_source['sha'],'sourceTree':latest_source['tree'],'sourceSnapshotHash':chain.digest(chain.snapshot_records(latest_source['files'],latest_source['modes'])),
                'abi':json.loads(latest_source['files']['docs/releases/v2-candidate.json'])['abi'],'packages':records}
        declared=re.search(r'^pub const VERSION: &str = "([0-9]+\.[0-9]+\.[0-9]+)";',latest_source['files']['crates/core/src/lib.rs'].decode('utf-8'),re.M)
        check(declared is not None and ledger['abi']==declared[1],'Published Farm ABI differs from actual source declaration')
        anchor={'verified':True,'chain':'release','ledgerSha256':chain.digest(ledger),'registryChecksums':checksums,
                **{key:ledger[key] for key in ['repositoryId','runId','runAttempt','sourceSHA','sourceTree']}}
        check(self.state()[0]==revision,'Publication state moved during baseline verification')
        chain.validate_baseline(ledger,anchor,'release',latest_source['files'],latest_source['modes'])
        return {'transaction':selected,'stateRevision':revision,'ledger':ledger,'anchor':anchor,'source':latest_source}
