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
WORKFLOW='.github/workflows/incremental-release.yaml'

class ReleaseBaselineReader:
    def __init__(self,transport,registry,read_source=evidence.source_reader):
        self.transport,self.registry,self.read_source=transport,registry,read_source
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
    def candidates(self,manifest,repository,last_attempt):
        run_id,first=manifest['producerRunId'],manifest['producerAttempt']
        check(type(last_attempt) is int and first<=last_attempt<first+16,'Candidate acceptance history exceeds bound')
        result={}
        expected={'ubuntu-24.04':{'os':'linux','arch':'x64','abi':'linux-x64-gnu'},
                  'macos-latest':{'os':'darwin','arch':'arm64','abi':'darwin-arm64'},
                  'windows-latest':{'os':'win32','arch':'x64','abi':'win32-x64-msvc'}}
        for platform in ['ubuntu-24.04','macos-latest','windows-latest']:
            for attempt in range(last_attempt,first-1,-1):
                run=self.transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(attempt))
                check(run['id']==run_id and run['run_attempt']==attempt and run['head_sha']==manifest['sourceSHA']
                      and run['path']==WORKFLOW and run['head_branch']=='main' and run['event'] in ['push','workflow_dispatch']
                      and run['repository']['id']==repository['id']==run['head_repository']['id'],'Candidate acceptance source or repository differs')
                try:
                    job=evidence.successful_job(execution.jobs_for(self.transport,run),'Independent release candidate consumer ('+platform+')')
                    check(job.get('run_id')==run_id,'Candidate consumer belongs to another run')
                    name='farm-release-candidate-consumer-'+manifest['sourceSHA']+'-'+platform+'-'+str(attempt)
                    report,artifact=execution.fetch_json_artifact(self.transport,run,name,'result.json')
                    execution.upload_binding(job,'Upload independent release candidate acceptance',artifact)
                    binding={'manifestHash':chain.digest(manifest),'sourceSHA':manifest['sourceSHA'],'sourceTree':manifest['sourceTree'],
                             'producerRunId':run_id,'producerAttempt':first,'consumerRunId':run_id,'consumerAttempt':attempt}
                    check(report.get('releaseBinding')==binding and report.get('status')=='passed'
                          and report.get('scratchRemoved') is True and report.get('cleanupConfirmed') is not False,'Candidate acceptance does not bind original frozen archives or cleanup')
                    check(report.get('platform')==expected[platform],'Candidate report OS or ABI differs')
                    result[platform]={'attempt':attempt,'artifactId':artifact['id'],'artifactDigest':artifact['digest']};break
                except (ValueError,KeyError):continue
            check(platform in result,'No valid independent candidate acceptance for '+platform)
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
        jobs=execution.jobs_for(self.transport,run)
        producer=evidence.successful_job(jobs,'Build independent release candidates')
        check(producer.get('run_id')==run_id,'Completed producer job belongs to another run')
        seal,artifact=execution.fetch_json_artifact(self.transport,run,'farm-release-seal-'+source['sha']+'-'+str(attempt))
        execution.upload_binding(producer,'Upload independent release seal',artifact)
        check(seal.get('manifestHash')==transaction and seal.get('profile')=='release' and seal.get('namespace')=='farm-release-v1'
              and seal.get('sourceTree')==source['tree'] and seal.get('producerJobId')==producer['id']
              and seal.get('packageInputs')==evidence.package_input_hashes(source),'Completed seal or source inputs differ')
        latest=self.transport.get('/actions/runs/'+str(run_id))
        check(type(latest['run_attempt']) is int and latest['run_attempt']>=attempt,'Latest release attempt differs')
        self.candidates(manifest,repository,latest['run_attempt'])
        accepted=False
        # Same frozen producer may finish in a later attempt; never borrow a
        # public result from another run or another source transaction.
        check(latest['run_attempt']-attempt<16,'Release recovery history exceeds bound')
        for current in range(latest['run_attempt'],attempt-1,-1):
            observed=self.transport.get('/actions/runs/'+str(run_id)+'/attempts/'+str(current))
            if observed['head_sha']!=source['sha'] or observed['path']!=WORKFLOW:continue
            try:
                public,meta=execution.fetch_json_artifact(self.transport,observed,'farm-release-public-acceptance-'+source['sha']+'-'+str(current))
                job=evidence.successful_job(execution.jobs_for(self.transport,observed),'Verify independent release public consumers')
                check(job.get('run_id')==run_id,'Completed public consumer belongs to another run')
                execution.upload_binding(job,'Upload independent release public acceptance',meta)
                check(public.get('manifestHash')==transaction and public.get('sourceTree')==source['tree']
                      and public.get('planHash')==manifest['planHash'] and public.get('producerJobId')==job['id'],'Completed public acceptance identity differs')
                expected={row['kind']+':'+row['name']:row['archiveSha256'] for row in manifest['packages']}
                check(public.get('packages')==expected and all(public.get(key) is True for key in ['registryInstall','nativeABI','latestTargets']),'Completed public consumer coverage differs')
                accepted=True;break
            except (ValueError,KeyError):continue
        check(accepted,'No independent public acceptance for completed manifest')
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
        ledger={'schemaVersion':1,'chain':'release','profile':'release','policy':chain.POLICY,'status':'success','registryVerified':True,
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
