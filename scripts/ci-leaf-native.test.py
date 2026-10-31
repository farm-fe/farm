"""First build -> trusted same-chain bundle -> actual leaf resolution/consumption."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest import mock


def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value


live=load('ci_leaf_integration','ci-leaf-native.py')
fixtures=load('ci_leaf_fixture','incremental-chain-plan.test.py')


class Contracts(unittest.TestCase):
    def setUp(self):
        files,family,identity=fixtures.fixture()
        files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
        files['rust-toolchain.toml']=b'[toolchain]\nchannel="nightly-2026-04-10"\n'
        files['docs/experiences/engineering.md']=b'Original engineering notes\n'
        package={'name':'@farmfe/core','version':'2.0.0','napi':{'targets':[]},'optionalDependencies':{}}
        for abi in live.chain.impact.TARGETS:
            name='@farmfe/core-'+abi;package['optionalDependencies'][name]='2.0.0'
            files['packages/core/npm/'+abi+'/package.json']=json.dumps({'name':name,'version':'2.0.0'}).encode()
        files['packages/core/package.json']=json.dumps(package).encode()
        self.origin={'sha':'a'*40,'tree':'b'*40,'files':files,'modes':{file:'100644' for file in files}}
        self.current=copy.deepcopy(self.origin);self.current.update({'sha':'c'*40,'tree':'d'*40})
        self.current['files']['docs/experiences/engineering.md']=b'Updated engineering notes\n'
        self.root=Path(tempfile.mkdtemp(prefix='farm-ci-byte-contract-'))
        self.native=b'\x7fELF\x02\x01'+b'\0'*12+b'\x3e\x00'+b'original native fixture'
        name='@farmfe/core-linux-x64-gnu';file='packages/core/binding/farm.linux-x64-gnu.node'
        target=self.root/file;target.parent.mkdir(parents=True);target.write_bytes(self.native)
        self.image='sha256:'+'e'*64
        self.captured={'originBuildSHA':self.origin['sha'],'records':[{'name':name,'file':file,'bytes':len(self.native),'sha256':hashlib.sha256(self.native).hexdigest()}],
            'runtimeCapture':{'profile':'ci','abi':live.ABI,'target':'x86_64-unknown-linux-gnu','errors':[],'runtime':{'node':'v22.20.0'},
                              'rustc':'rustc measured nightly','cargo':'cargo measured nightly','activeToolchain':'nightly-2026-04-10-x86_64-unknown-linux-gnu (directory override)',
                              'linker':{'version':'gcc measured version'},'flagsHash':live.requested_flags(),'unboundBuildVariables':[]},
            'container':{'containerImageId':self.image,'buildExitCode':0,'cleanupConfirmed':True,'timedOut':False}}
        self.producer_env={'GITHUB_ACTIONS':'true','GITHUB_EVENT_NAME':'push','GITHUB_REF':'refs/heads/main','GITHUB_REPOSITORY':'farm-fe/farm',
                           'GITHUB_WORKFLOW_REF':'farm-fe/farm/.github/workflows/ci.yaml@refs/heads/main','GITHUB_SHA':self.origin['sha'],
                           'GITHUB_RUN_ID':'1','GITHUB_RUN_ATTEMPT':'1','GITHUB_REPOSITORY_ID':'5'}
        self.environment={**self.producer_env,'GITHUB_EVENT_NAME':'pull_request','GITHUB_SHA':self.current['sha'],'GITHUB_RUN_ID':'9'}
        self.manifest,self.bodies=live.create_bundle(self.origin,self.captured,self.root,self.producer_env)
        self.run={'id':1,'run_attempt':1,'head_sha':self.origin['sha'],'path':live.WORKFLOW,'event':'push','head_branch':'main','status':'completed','conclusion':'success',
                  'repository':{'id':5},'head_repository':{'id':5}}
        self.job={'id':2,'run_id':1,'name':'Primary Linux / native / '+live.SUFFIX,'status':'completed','conclusion':'success',
                  'steps':[{'name':live.UPLOAD,'status':'completed','conclusion':'success','started_at':'2026-01-01T00:00:00Z','completed_at':'2026-01-01T00:02:00Z'}]}
        self.main=self.origin['sha'];self.runs=[{'id':1,'run_attempt':1}]
        self.transport=mock.Mock();self.transport.get.side_effect=self.get;self.transport.archive.side_effect=lambda number:self.body
        self.pack()

    def pack(self):
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as archive:
            archive.writestr('bundle.json',json.dumps(self.manifest))
            for name,body in self.bodies.items():archive.writestr(name,body)
        self.body=output.getvalue()
        self.artifact={'id':8,'name':'farm-ci-byte-bundle-'+self.origin['sha']+'-'+live.ABI+'-1','workflow_run':{'id':1},'expired':False,
                       'created_at':'2026-01-01T00:01:00Z','size_in_bytes':len(self.body),'digest':'sha256:'+hashlib.sha256(self.body).hexdigest()}

    def get(self,route):
        if route=='':return {'id':5,'default_branch':getattr(self,'default_branch','main')}
        if route.startswith('/actions/workflows/ci.yaml/runs?'):return {'workflow_runs':self.runs}
        if route=='/actions/runs/1/attempts/1':return self.run
        if route.startswith('/actions/runs/1/artifacts?'):return {'total_count':1,'artifacts':[self.artifact]}
        if route=='/actions/artifacts/8':return self.artifact
        if route.startswith('/actions/runs/1/attempts/1/jobs?'):return {'jobs':[self.job]}
        if route=='/branches/main':return {'commit':{'sha':self.main}}
        raise AssertionError('Unexpected GitHub proof route')

    def schedule(self,source=None,trusted=None,image=None):
        return live.schedule(self.transport,source or self.current,trusted or self.origin,self.environment,
                             read_source=lambda sha:self.origin,measure_image=lambda:image or self.image)

    def test_first_build_then_trusted_main_producer_then_non_full_matrix_and_actual_byte_consumption(self):
        self.runs=[]
        first,receipt,bodies=self.schedule();self.assertEqual(first['nativeMode'],'build');self.assertEqual(set(first['nativeBuildMatrix']),set(live.chain.impact.TARGETS))
        self.assertIsNone(receipt);self.assertEqual(bodies,{})
        self.runs=[{'id':1,'run_attempt':1}]
        second,receipt,bodies=self.schedule()
        self.assertEqual(second['nativeMode'],'reuse');self.assertEqual(set(second['nativeBuildMatrix']),set(live.chain.impact.TARGETS)-{live.ABI})
        self.assertEqual(second['tsMatrix'],live.chain.impact.PRIMARY);self.assertEqual(second['examplesMatrix'],live.chain.impact.PRIMARY)
        self.assertFalse(receipt['acceptanceInherited']);self.assertEqual(receipt['originBuildSHA'],self.origin['sha']);self.assertEqual(receipt['consumerSHA'],self.current['sha'])
        self.transport.archive.assert_called_once_with(8)
        materialized=self.root/'materialized';live.materialize(receipt,bodies,materialized)
        commitment=hashlib.sha256((materialized/'consumer.json').read_bytes()).hexdigest()
        destination=self.root/'consumer';destination.mkdir()
        live.consume(materialized,commitment,self.current,self.environment,destination)
        self.assertEqual((destination/'packages/core/binding/farm.linux-x64-gnu.node').read_bytes(),self.native)

    def test_non_leaf_source_chmod_unknown_policy_and_lock_change_rebuild(self):
        for change in ['source','mode','policy','lock']:
            source=copy.deepcopy(self.current)
            if change=='source':source['files']['crates/core/src/lib.rs']+=b'// changed'
            elif change=='mode':source['modes']['docs/experiences/engineering.md']='100755'
            elif change=='policy':source['files']['scripts/unknown-policy.py']=b'new policy';source['modes']['scripts/unknown-policy.py']='100644'
            else:source['files']['Cargo.lock']=b'new lock';source['modes']['Cargo.lock']='100644'
            decision,receipt,_=self.schedule(source=source);self.assertEqual(decision['nativeMode'],'build');self.assertIsNone(receipt)

    def test_release_namespace_wrong_profile_image_or_toolchain_rebuilds(self):
        original=copy.deepcopy(self.manifest)
        for key,value in [('chain','release'),('context.profile','release'),('context.activeToolchain','nightly-other')]:
            self.manifest=copy.deepcopy(original)
            if '.' in key:self.manifest['context'][key.split('.')[1]]=value
            else:self.manifest[key]=value
            self.pack();self.assertEqual(self.schedule()[0]['nativeMode'],'build')
        self.manifest=original;self.pack();self.assertEqual(self.schedule(image='sha256:'+'0'*64)[0]['nativeMode'],'build')

    def test_failed_cancelled_or_missing_upload_producer_cannot_be_reused(self):
        for status,conclusion in [('completed','failure'),('completed','cancelled'),('in_progress',None)]:
            self.job.update({'status':status,'conclusion':conclusion});self.assertEqual(self.schedule()[0]['nativeMode'],'build')
        self.job.update({'status':'completed','conclusion':'success','steps':[]});self.assertEqual(self.schedule()[0]['nativeMode'],'build')

    def test_aggregate_failure_does_not_inherit_acceptance_but_successful_producer_bytes_remain_verifiable(self):
        self.run['conclusion']='failure'
        decision,receipt,_=self.schedule();self.assertEqual(decision['nativeMode'],'reuse');self.assertFalse(receipt['acceptanceInherited'])
        outcomes={name:{'result':'success'} for name in ['type-check','primary-linux','primary-macos','primary-windows','call-rust-build-release','check-native-candidate-assembly','examples-test','ts-test']}
        self.assertEqual(len(live.check_execution('reuse',outcomes)['nativeBuildMatrix']),8)
        for result in ['skipped','failure','cancelled']:
            outcomes['examples-test']['result']=result
            with self.assertRaises(ValueError):live.check_execution('reuse',outcomes)

    def test_wrong_zip_digest_missing_member_or_partial_inventory_rebuilds(self):
        self.artifact['digest']='sha256:'+'0'*64;self.assertEqual(self.schedule()[0]['nativeMode'],'build')
        self.pack();self.manifest['records']=[];self.pack();self.assertEqual(self.schedule()[0]['nativeMode'],'build')

    def test_moving_main_or_access_denial_falls_back_to_build(self):
        self.main='0'*40;self.assertEqual(self.schedule()[0]['nativeMode'],'build')
        self.main=self.origin['sha'];self.transport.archive.side_effect=PermissionError('denied')
        self.assertEqual(self.schedule()[0]['nativeMode'],'build')

    def test_changed_consumer_binding_attempt_or_bytes_are_rejected(self):
        _,receipt,bodies=self.schedule();materialized=self.root/'consumer-bytes';live.materialize(receipt,bodies,materialized)
        commitment=hashlib.sha256((materialized/'consumer.json').read_bytes()).hexdigest();destination=self.root/'target';destination.mkdir()
        with self.assertRaisesRegex(ValueError,'source/run/attempt'):
            live.consume(materialized,commitment,self.current,{**self.environment,'GITHUB_RUN_ATTEMPT':'2'},destination)
        path=materialized/'packages/core/binding/farm.linux-x64-gnu.node';path.write_bytes(b'corrupted')
        with self.assertRaisesRegex(ValueError,'bytes differ'):live.consume(materialized,commitment,self.current,self.environment,destination)

    def test_main_documentation_leaf_can_use_older_main_bytes_without_creating_new_origin(self):
        self.environment['GITHUB_EVENT_NAME']='push';self.main=self.current['sha']
        decision,receipt,_=self.schedule(trusted=self.current)
        self.assertEqual(decision['nativeMode'],'reuse');self.assertEqual(receipt['originBuildSHA'],self.origin['sha'])

    def test_other_default_branch_falls_back_without_reading_native_archive(self):
        self.default_branch='other'
        decision,receipt,_=self.schedule()
        self.assertEqual(decision['nativeMode'],'build');self.assertIsNone(receipt)
        self.transport.archive.assert_not_called()

    def test_missing_final_symlink_target_is_rejected_before_exclusive_write(self):
        _,receipt,bodies=self.schedule()
        materialized=self.root/'materialized';live.materialize(receipt,bodies,materialized)
        commitment=hashlib.sha256((materialized/'consumer.json').read_bytes()).hexdigest()
        original=Path.is_symlink
        for pool in [False,True]:
            destination=self.root/('pool-consumer' if pool else 'consumer');destination.mkdir()
            def final_link(value):
                return value.name=='farm.linux-x64-gnu.node' and any(part in ['consumer','pool-consumer'] for part in value.parts) or original(value)
            # Model a dangling final link without requiring Windows link privileges.
            with mock.patch.object(Path,'is_symlink',final_link):
                with self.assertRaisesRegex(ValueError,'pre-existing consumer native paths'):
                    live.consume(materialized,commitment,self.current,self.environment,destination,pool=pool)
            self.assertEqual(list(destination.rglob('*.node')),[])

    def test_changed_flags_or_unbound_build_environment_cannot_authorize_reuse(self):
        for field,value in [('flagsHash','f'*64),('unboundBuildVariables',['RUSTUP_TOOLCHAIN'])]:
            captured=copy.deepcopy(self.captured);captured['runtimeCapture'][field]=value
            with self.assertRaisesRegex(ValueError,'flags or environment'):
                live.create_bundle(self.origin,captured,self.root,self.producer_env)
        self.manifest['context']['flagsHash']='f'*64;self.pack()
        decision,receipt,_=self.schedule()
        self.assertEqual(decision['nativeMode'],'build');self.assertIsNone(receipt)

    def test_actual_workflow_wiring_binds_native_choice_consumers_and_existing_required_contexts(self):
        root=Path(__file__).parent.parent
        workflow=(root/'.github/workflows/ci.yaml').read_text(encoding='utf-8')
        primary=(root/'.github/workflows/ci-primary-platform.yaml').read_text(encoding='utf-8')
        producer=(root/'.github/workflows/rust-build-user.yaml').read_text(encoding='utf-8')
        self.assertIn('native_mode: ${{ needs.type-check.outputs.linux_native_mode',workflow)
        self.assertIn('if: inputs.native_mode != \'reuse\'',primary)
        self.assertEqual(primary.count('if: always() && needs.native-disposition.result == \'success\''),2)
        self.assertIn('actually load the GNU native binding',primary)
        self.assertIn("inputs.profile == 'ci'",producer);self.assertIn("github.event_name == 'push'",producer)
        self.assertIn('Upload trusted CI native byte bundle',producer)
        self.assertIn('needs: [examples-test, incremental-execution-gate]',workflow)
        self.assertIn('needs: [ts-test, incremental-execution-gate]',workflow)
        self.assertIn('ci-leaf-native.py gate --mode',workflow)


if __name__=='__main__':unittest.main(verbosity=2)
