"""Cargo source-only preparation and immutable publication contracts; no network."""
import copy
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile
from unittest import mock
spec=importlib.util.spec_from_file_location('source_release',Path(__file__).with_name('cargo-source-release.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Contracts(unittest.TestCase):
    def fixture(self):
        root=Path(tempfile.mkdtemp());self.addCleanup(__import__('shutil').rmtree,root);(root/'files').mkdir()
        identity={'repository':'farm/farm','repositoryId':1,'runId':2,'sourceSHA':'a'*40,'sourceTree':'b'*40,'pr':3,'headSHA':'c'*40,'baseSHA':'d'*40,'workflow':m.WORKFLOW}
        records=[];selection=[]
        for name,deps in [('leaf',[]),('consumer',['leaf'])]:
            stream=io.BytesIO()
            with tarfile.open(fileobj=stream,mode='w:gz') as archive:
                body=f'[package]\nname="{name}"\nversion="1.0.0"\n'.encode();row=tarfile.TarInfo(name+'-1.0.0/Cargo.toml');row.size=len(body);archive.addfile(row,io.BytesIO(body))
            archive=stream.getvalue();metadata=json.dumps({'name':name,'vers':'1.0.0'}).encode();record={'name':name,'version':'1.0.0','dependencies':deps}
            for key,body,ext in [('archive',archive,'.crate'),('metadata',metadata,'.json')]:
                file=name+ext;(root/'files'/file).write_bytes(body);record.update({key:file,key+'Bytes':len(body),key+'Sha256':m.digest(body)})
            records.append(record);selection.append({'name':name,'version':'1.0.0'})
        receipt={'identity':identity,'verified':True,'producerAttempt':1,'selection':selection,'selectionHash':m.digest(m.encoded(selection)),'records':records}
        (root/'receipt.json').write_bytes(m.encoded(receipt));return root,identity,receipt
    def test_complete_frozen_inventory_requires_identity_and_exact_bytes(self):
        root,identity,receipt=self.fixture();self.assertEqual(m.verify_frozen(root,identity),receipt)
        with self.assertRaisesRegex(ValueError,'identity'):m.verify_frozen(root,{**identity,'runId':3})
        (root/'files'/receipt['records'][0]['archive']).write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError,'bytes'):m.verify_frozen(root,identity)
    def test_dependency_order_and_extra_crates_cannot_be_self_asserted(self):
        root,identity,receipt=self.fixture();receipt['records'].reverse();(root/'receipt.json').write_bytes(m.encoded(receipt))
        with self.assertRaisesRegex(ValueError,'order'):m.verify_frozen(root,identity)
    def test_conflict_anywhere_stops_before_first_upload(self):
        root,identity,receipt=self.fixture();registry=mock.Mock();registry.lookup.side_effect=[None,'bad']
        with self.assertRaisesRegex(ValueError,'conflict'):m.publish(root,identity,registry)
        registry.upload.assert_not_called()
    def test_partial_failure_retries_same_frozen_bytes_and_dependency_order(self):
        root,identity,receipt=self.fixture();published={};attempts=[];fail=True
        def upload(metadata,archive):
            nonlocal fail
            name=json.loads(metadata)['name'];attempts.append((name,archive))
            if name=='consumer' and fail:fail=False;raise RuntimeError('network failure')
            published[name]=m.digest(archive)
        registry=mock.Mock();registry.lookup.side_effect=lambda row:published.get(row['name']);registry.upload.side_effect=upload
        with self.assertRaisesRegex(RuntimeError,'network'):m.publish(root,identity,registry,sleep=lambda _:None)
        self.assertEqual(json.loads((root/'publication.json').read_text())['packages'][-1]['status'],'failed')
        result=m.publish(root,identity,registry,sleep=lambda _:None)
        self.assertEqual([name for name,_ in attempts],['leaf','consumer','consumer']);self.assertEqual(attempts[1][1],attempts[2][1]);self.assertTrue(all(row['status']=='verified' for row in result['packages']))
    def test_registry_read_back_is_mandatory(self):
        root,identity,_=self.fixture();registry=mock.Mock();registry.lookup.return_value=None
        with self.assertRaisesRegex(ValueError,'confirm'):m.publish(root,identity,registry,sleep=lambda _:None)
        self.assertEqual(registry.upload.call_count,1)
    def test_original_merge_trigger_and_tree_are_required(self):
        env={'GITHUB_EVENT_NAME':'pull_request','GITHUB_REPOSITORY':'farm/farm','GITHUB_REPOSITORY_ID':'1','GITHUB_RUN_ID':'2'}
        event={'action':'closed','number':3,'pull_request':{'merged':True,'merge_commit_sha':'a'*40,'base':{'ref':'main','sha':'d'*40,'repo':{'full_name':'farm/farm'}},'head':{'ref':'changeset-release/main','sha':'c'*40,'repo':{'full_name':'farm/farm'}}}}
        git=lambda *args:'a'*40 if args[-1]=='HEAD' else 'b'*40
        self.assertEqual(m.identity(event,env,git)['runId'],2)
        for key,value in [('merged',False),('merge_commit_sha','e'*40)]:
            changed=copy.deepcopy(event);changed['pull_request'][key]=value
            with self.assertRaises(ValueError):m.identity(changed,env,git)
        for where,key,value in [('base','ref','other'),('head','ref','feature'),('head','repo',{'full_name':'fork/farm'})]:
            changed=copy.deepcopy(event);changed['pull_request'][where][key]=value
            with self.assertRaises(ValueError):m.identity(changed,env,git)
    def test_main_push_identity_needs_no_pr_history_and_rejects_source_drift(self):
        sha='a'*40
        env={'GITHUB_EVENT_NAME':'push','GITHUB_REF':'refs/heads/main','GITHUB_SHA':sha,'GITHUB_WORKFLOW_SHA':sha,'GITHUB_REPOSITORY':'farm/farm','GITHUB_REPOSITORY_ID':'1','GITHUB_RUN_ID':'2','GITHUB_WORKFLOW_REF':'farm/farm/'+m.WORKFLOW+'@refs/heads/main'}
        event={'ref':'refs/heads/main','after':sha,'forced':False,'deleted':False}
        git=lambda *args:sha if args[-1]=='HEAD' else 'b'*40
        result=m.identity(event,env,git)
        self.assertEqual(result['event'],'push');self.assertNotIn('pr',result)
        for key,value in [('ref','refs/heads/feature'),('after','c'*40),('forced',True),('deleted',True)]:
            with self.assertRaises(ValueError):m.identity({**event,key:value},env,git)
        for key,value in [('GITHUB_WORKFLOW_SHA','c'*40),('GITHUB_WORKFLOW_REF','fork/farm/'+m.WORKFLOW+'@refs/heads/main')]:
            with self.assertRaises(ValueError):m.identity(event,{**env,key:value},git)
        with self.assertRaises(ValueError):m.identity(event,env,lambda *args:'d'*40)

    def test_archive_path_and_link_are_rejected(self):
        for name,kind in [('leaf-1.0.0/../../escape',tarfile.REGTYPE),('leaf-1.0.0/link',tarfile.SYMTYPE)]:
            stream=io.BytesIO()
            with tarfile.open(fileobj=stream,mode='w:gz') as archive:
                row=tarfile.TarInfo(name);row.type=kind;row.linkname='outside';archive.addfile(row)
            with self.assertRaises(ValueError):m.safe_archive(stream.getvalue(),'leaf','1.0.0')
    def test_missing_deleted_or_omitted_original_artifact_blocks_retry(self):
        root,identity,_=self.fixture()
        with mock.patch.object(m,'api',return_value={'artifacts':[]}),mock.patch.dict(m.os.environ,{'GITHUB_RUN_ATTEMPT':'2'}):
            with self.assertRaisesRegex(ValueError,'identity is unproven'):m.restore(root/'missing',identity)
            self.assertFalse((root/'missing').exists())
        with mock.patch.object(m,'api',return_value={'artifacts':[]}),mock.patch.dict(m.os.environ,{'GITHUB_RUN_ATTEMPT':'1'}):
            self.assertFalse(m.restore(root/'first',identity))
    def test_registry_redirect_cannot_forward_publication_credentials(self):
        with self.assertRaisesRegex(ValueError,'redirects'):
            m.NoRedirect().redirect_request(None,None,302,'redirect',{},'https://untrusted.invalid')
    def test_extracted_manifest_and_readme_must_match_frozen_archive(self):
        root,_,_=self.fixture();normalized=root/'Cargo.toml';normalized.write_bytes(b'original');(root/'README.md').write_bytes(b'readme')
        files={'leaf-1.0.0/Cargo.toml':b'original','leaf-1.0.0/README.md':b'readme'};pkg={'name':'leaf','version':'1.0.0','readme':'README.md'}
        self.assertEqual(m.bind_normalized_metadata(files,pkg,normalized),b'readme')
        normalized.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError,'manifest differs'):m.bind_normalized_metadata(files,pkg,normalized)
        normalized.write_bytes(b'original');(root/'README.md').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError,'README differs'):m.bind_normalized_metadata(files,pkg,normalized)
    def test_original_successful_attempt_restores_after_later_failure_and_rejects_bad_proofs(self):
        for variant in ['valid','expired','digest','failed-job','failed-package','wrong-source','wrong-workflow','wrong-attempt','unsigned','wrong-signed-id']:
            with self.subTest(variant=variant):
                root,identity,receipt=self.fixture()
                identity.update(repository='farm-fe/farm',repositoryId=507542208,event='push')
                if variant=='wrong-attempt':receipt['producerAttempt']=9
                (root/'receipt.json').write_bytes(m.encoded(receipt))
                buffer=io.BytesIO()
                with zipfile.ZipFile(buffer,'w') as archive:
                    for file in [root/'receipt.json',*(root/'files').iterdir()]:archive.writestr(str(file.relative_to(root)),file.read_bytes())
                body=buffer.getvalue();digest='sha256:'+m.digest(body)
                artifact={'id':4,'name':'cargo-source-2','expired':variant=='expired','workflow_run':{'id':2},'size_in_bytes':len(body),'digest':digest}
                if variant=='digest':artifact['digest']='sha256:'+'0'*64
                run={'id':2,'run_attempt':1,'path':m.WORKFLOW,'event':'push','head_branch':'main','head_sha':identity['sourceSHA'],
                     'repository':{'id':identity['repositoryId'],'full_name':'farm-fe/farm'},'head_repository':{'id':identity['repositoryId']}}
                if variant=='wrong-source':run['head_sha']='0'*40
                if variant=='wrong-workflow':run['path']='other.yaml'
                steps=[{'name':name,'status':'completed','conclusion':'success'} for name in ['Verify fixed main versions and Changesets readiness','Package and verify exact source crates','Freeze verified Cargo source archives']]
                if variant=='failed-package':steps[1]['conclusion']='failure'
                job={'id':7,'run_id':2,'run_attempt':1,'head_sha':run['head_sha'],'name':'Prepare Cargo source archives','status':'completed','conclusion':'failure' if variant=='failed-job' else 'success','steps':steps}
                def api(route):
                    if '/artifacts?' in route:
                        if 'name=farm-release-bindings-' in route:
                            found=[] if variant=='unsigned' or not route.endswith('-1') else [{'id':5}]
                            return {'artifacts':found,'total_count':len(found)}
                        return {'artifacts':[artifact]}
                    if route.endswith('/artifacts/4'):return artifact
                    if '/jobs?' in route:return {'jobs':[job]}
                    attempt=int(route.rsplit('/attempts/',1)[1]);return {**run,'run_attempt':attempt}
                def verified_fixture(transport,execution,source,scope):
                    self.assertEqual(scope,'cargo')
                    expected={'runId':2,'attempt':1,'callerWorkflow':m.WORKFLOW,'sourceSHA':identity['sourceSHA'],'repositoryId':identity['repositoryId']}
                    row={'artifactId':99 if variant=='wrong-signed-id' else 4,'artifactDigest':digest,'artifactName':'cargo-source-2',
                         'originAttempt':1,'jobId':7,'producer':'Prepare Cargo source archives','uploadStep':'Freeze verified Cargo source archives'}
                    return expected,{'cargo':row}
                with mock.patch.object(m,'api',side_effect=api),mock.patch.object(m.subprocess,'check_output',return_value=body),mock.patch.dict(m.os.environ,{'GITHUB_RUN_ATTEMPT':'3'}):
                    destination=root/'restored';binding={}
                    if variant=='valid':
                        self.assertTrue(m.restore(destination,identity,proof_provider=verified_fixture,binding_output=binding))
                        self.assertEqual(m.verify_frozen(destination,identity),receipt)
                        self.assertEqual(binding,{'artifact_id':'4','artifact_digest':digest[7:],'producer_attempt':'1'})
                    else:
                        with self.assertRaises(ValueError):m.restore(destination,identity,proof_provider=verified_fixture)
    def test_source_workflow_keeps_original_trigger_and_publisher_is_no_build(self):
        source=(m.ROOT/'.github/workflows/cargo-source-release.yaml').read_text();publisher=source.split('  publish:')[1]
        self.assertIn('  push:\n    branches: [main]',source);self.assertNotIn('types: [closed]',source);self.assertIn('group: farm-stable-release',source)
        self.assertNotIn('CARGO_REGISTRY_TOKEN',source.split('  publish:')[0]);self.assertNotIn('npm',publisher);self.assertNotIn('cargo package',publisher)
        code=Path(m.__file__).read_text();self.assertNotIn('--no-verify',code);self.assertIn("'cargo','package','--locked','--offline'",code)
    def test_prepared_family_freezes_retained_versions_and_rechecks_before_writes(self):
        root,identity,receipt=self.fixture()
        retained={'name':'already-published','version':'2.0.0','sha256':'f'*64}
        family=[*receipt['selection'],{'name':retained['name'],'version':retained['version']}]
        receipt.update(preparedFamily=family,preparedFamilyHash=m.digest(m.encoded(family)),retained=[retained],retainedHash=m.digest(m.encoded([retained])))
        (root/'receipt.json').write_bytes(m.encoded(receipt));m.verify_frozen(root,identity)
        registry=mock.Mock();registry.lookup.return_value='bad'
        with self.assertRaisesRegex(ValueError,'Retained Cargo'):m.publish(root,identity,registry)
        registry.upload.assert_not_called()
        receipt['retained'][0]['sha256']='a'*64
        (root/'receipt.json').write_bytes(m.encoded(receipt))
        with self.assertRaisesRegex(ValueError,'Retained Cargo receipt hash'):m.verify_frozen(root,identity)
    def test_prepared_family_cannot_omit_or_republish_retained_member(self):
        root,identity,receipt=self.fixture()
        retained={'name':'leaf','version':'1.0.0','sha256':'f'*64}
        receipt.update(preparedFamily=receipt['selection'],preparedFamilyHash=m.digest(m.encoded(receipt['selection'])),retained=[retained],retainedHash=m.digest(m.encoded([retained])))
        (root/'receipt.json').write_bytes(m.encoded(receipt))
        with self.assertRaisesRegex(ValueError,'cannot be republished'):m.verify_frozen(root,identity)
    def test_missing_credentials_fail_before_restore_or_upload(self):
        event={'pull_request':{}}
        with mock.patch.object(m,'identity',return_value={}),mock.patch.object(m.Path,'read_text',return_value=json.dumps(event)),mock.patch.dict(m.os.environ,{'GITHUB_EVENT_PATH':'fixture','CARGO_REGISTRY_TOKEN':''}),mock.patch.object(m.sys,'argv',['script','publish']),mock.patch.object(m,'restore') as restore:
            with self.assertRaisesRegex(ValueError,'secret required'):m.main()
            restore.assert_not_called()
    def test_real_retry_helper_allows_only_proven_pre_freeze_source_failure(self):
        root,identity,_=self.fixture();base='repos/'+identity['repository'];attempt=base+'/actions/runs/2/attempts/1'
        run={'id':2,'run_attempt':1,'path':m.WORKFLOW,'head_sha':identity['sourceSHA'],'status':'completed','event':'pull_request','repository':{'id':1},'head_repository':{'id':1}}
        jobs=[{'run_id':2,'name':'Prepare Cargo source archives','status':'completed','conclusion':'failure','steps':[{'name':'Freeze verified Cargo source archives','status':'completed','conclusion':'skipped'}]},
              {'run_id':2,'name':'Publish frozen Cargo source archives','status':'completed','conclusion':'skipped','steps':[]}]
        def api(route):
            if route==base:return {'id':1}
            if route==base+'/actions/runs/2/artifacts?per_page=100':return {'artifacts':[]}
            if route==attempt:return run
            if route==attempt+'/jobs?per_page=100':return {'total_count':2,'jobs':jobs}
            raise AssertionError(route)
        with mock.patch.object(m,'api',side_effect=api),mock.patch.dict(m.os.environ,{'GITHUB_RUN_ATTEMPT':'2'}):
            self.assertFalse(m.restore(root/'missing',identity));self.assertFalse((root/'missing').exists())
            jobs[1]['conclusion']='failure'
            with self.assertRaisesRegex(ValueError,'publisher may have executed'):m.restore(root/'missing',identity)
    def test_cargo_artifact_inventory_must_be_complete(self):
        root,identity,_=self.fixture()
        with mock.patch.object(m,'api',return_value={'artifacts':[{'name':'other'}]*100}),mock.patch.dict(m.os.environ,{'GITHUB_RUN_ATTEMPT':'1'}):
            with self.assertRaisesRegex(ValueError,'verification bound'):m.restore(root/'missing',identity)
if __name__=='__main__':unittest.main(verbosity=2)
