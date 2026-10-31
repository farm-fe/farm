"""Functional CI proof is source-tree acceptance, never build-byte reuse."""
# cspell:words importlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest import mock
import zipfile

spec = importlib.util.spec_from_file_location("functional_contract", Path(__file__).with_name("incremental-release-functional.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


class Contracts(unittest.TestCase):
    def fixture(self):
        sha, tree = "a" * 40, "b" * 40
        bindings = {"ci": {"runId": 10, "attempt": 1, "testedSHA": sha}, "unit": {"runId": 20, "attempt": 1, "testedSHA": sha}}
        bindings.update({key: {"runId": number, "attempt": 1} for key, number in [('lint',30),('spell',40),('title',50)]})
        runs = {number: {"id": number, "run_attempt": 1, "path": path, "status": "completed", "conclusion": "success",
                        "repository": {"id": 5}, "head_repository": {"id": 5}, "event": "push", "head_branch": "main", "head_sha": sha}
                for number, path in [(10, '.github/workflows/ci.yaml'), (20, '.github/workflows/rust-test.yaml'),
                                     (30,'.github/workflows/lint.yaml'),(40,'.github/workflows/code-spell-check.yaml'),(50,'.github/workflows/lint-pr-title.yml')]}
        plan = {"testedSHA": sha, "testedTree": tree, "runId": "10", "runAttempt": "1", "repository": "farm-fe/farm", "baseSHA": "c"*40}
        jobs = {}
        def archive(number):
            value = {**plan, "runId": str(number)}
            value['planHash'] = module.entry.chain.digest(value)
            output = io.BytesIO()
            with zipfile.ZipFile(output, 'w') as file: file.writestr('plan.json', json.dumps(value))
            return output.getvalue()
        bodies = {number: archive(number) for number in runs}
        summary = {'schemaVersion': 2, 'protocol': 'commerce-business-2048-farm-supervised-v2', 'runId': 10, 'runAttempt': 1,
                   'repositoryId': 5, 'event': 'push', 'headSHA': sha, 'baseSHA': 'c'*40, 'prNumber': None,
                   'reports': [{'metadata': {'os': os, 'arch': arch, 'revision': sha, 'baseSHA': 'c'*40, 'fixtureSha256': 'd'*64, 'lockSha256': 'e'*64},
                                'status': 'complete', 'cells': [{'cacheMode': mode, 'samples': 3, 'validSamples': 3, 'medianMs': 100, 'iqrMs': 2} for mode in ['cold','warm']]}
                               for os,arch in [('linux','x64'),('darwin','arm64'),('win32','x64')]]}
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as file:file.writestr('summary.json',json.dumps(summary))
        bodies[110]=output.getvalue()
        artifacts = {number: {"id": number, "name": 'impact-plan-'+sha+'-1', "expired": False, "workflow_run": {"id": number},
                             "created_at":"2", "size_in_bytes": len(bodies[number]), "digest": 'sha256:'+hashlib.sha256(bodies[number]).hexdigest()} for number in runs}
        artifacts[110] = {'id':110,'name':'performance-pr-summary-1','expired':False,'workflow_run':{'id':10},
                          'created_at':'2','size_in_bytes':len(bodies[110]),'digest':'sha256:'+hashlib.sha256(bodies[110]).hexdigest()}
        for number, names in [(10, ['Incremental CI plan (shadow only)', *module.FUNCTIONAL]),
                              (20, ['Incremental Unit CI plan (shadow only)', 'Check', *['Test - '+name for name in module.PLATFORMS]])]:
            jobs[number] = [{"id":index+number*100, "run_id":number, "name": name, "status": "completed", "conclusion": "success", "started_at":"1", "completed_at":"3", "steps":[{"name":"Upload fixed source impact plan","status":"completed","conclusion":"success","started_at":"1","completed_at":"3"}]} for index,name in enumerate(names)]
        def get(route):
            if route == '': return {"id": 5, "full_name": "farm-fe/farm", "default_branch": "main"}
            if route.startswith('/actions/runs/'):
                number=int(route.split('/')[3])
                if '/artifacts?' in route: return {"total_count": 1, "artifacts": [artifacts[110 if 'performance-pr-summary-' in route else number]]}
                return runs[number]
            if route.startswith('/git/commits/'): return {"tree": {"sha": tree}}
            if route.startswith('/actions/artifacts/'): return artifacts[int(route.split('/')[-1])]
            raise AssertionError('Unexpected proof API route')
        transport = mock.Mock(); transport.get.side_effect=get; transport.archive.side_effect=lambda number:bodies[number]
        return {"sha": sha,"tree":tree}, bindings, transport, jobs, runs, artifacts

    def verify(self, data):
        source, bindings, transport, jobs, runs, artifacts=data
        with mock.patch.object(module.entry.execution, 'jobs_for', side_effect=lambda client,run:jobs[run['id']]):
            return module.verify_functional(transport, source, bindings)

    def test_exact_functional_and_unit_evidence_keeps_nineteen_roles_and_no_build_bytes(self):
        self.assertEqual(len(module.FUNCTIONAL)+4,19)
        result=self.verify(self.fixture());self.assertTrue(result['functionalAcceptanceVerified']);self.assertFalse(result['buildBytesReused'])

    def test_skipped_docs_or_missing_performance_is_not_success(self):
        data=self.fixture();data[3][10]=[row for row in data[3][10] if row['name']!='Commerce Performance (macos-latest)']
        with self.assertRaises(ValueError):self.verify(data)
        data=self.fixture();next(row for row in data[3][10] if row['name']=='Docs / windows-latest')['conclusion']='skipped'
        with self.assertRaises(ValueError):self.verify(data)

    def test_different_tree_fork_or_attempt_is_rejected(self):
        data=self.fixture();data[0]['tree']='c'*40
        with self.assertRaisesRegex(ValueError,'tested functional tree'):self.verify(data)
        for field,value in [('head_repository',{'id':99}),('run_attempt',2),('conclusion','failure')]:
            data=self.fixture();data[4][10][field]=value
            with self.assertRaises(ValueError):self.verify(data)

    def test_wrong_artifact_bytes_cannot_supply_source_proof(self):
        data=self.fixture();data[5][10]['digest']='sha256:'+'f'*64
        with self.assertRaisesRegex(ValueError,'digest differs'):self.verify(data)

    def test_missing_bindings_do_not_authorize_release(self):
        data=self.fixture();data[1].pop('unit')
        with self.assertRaisesRegex(ValueError,'Exact CI, Unit'):self.verify(data)

    def test_run3_resolves_ci_and_unit_plans_from_actual_planner_attempt1(self):
        data=self.fixture();source,bindings,transport,jobs,runs,artifacts=data
        old_get=transport.get.side_effect
        for scope in ['ci','unit']:bindings[scope]['attempt']=3;runs[bindings[scope]['runId']]['run_attempt']=3
        summary=json.loads(zipfile.ZipFile(io.BytesIO(transport.archive(110))).read('summary.json'));summary['runAttempt']=3
        # Schema2 cannot authorize a mixed-attempt performance summary; this test
        # isolates planner provenance while the real schema3 chain is checked separately.
        def get(route):
            value=old_get(route)
            if '/attempts/' in route:return {**value,'run_attempt':int(route.split('/attempts/')[1].split('/')[0])}
            return value
        transport.get.side_effect=get
        with mock.patch.object(module,'verify_performance',return_value=summary):
            self.assertTrue(self.verify(data)['functionalAcceptanceVerified'])
        self.assertTrue(any('impact-plan-' in call.args[0] and call.args[0].endswith('-1') for call in transport.get.call_args_list))
        self.assertFalse(any('impact-plan-' in call.args[0] and call.args[0].endswith('-3') for call in transport.get.call_args_list))

    def test_main_acceptance_does_not_depend_on_historical_pull_request_associations(self):
        data=self.fixture()
        for run in data[4].values():run['pull_requests']=[]
        data[1].pop('title')
        self.assertTrue(self.verify(data)['functionalAcceptanceVerified'])

    def test_performance_bytes_are_checked_not_just_job_names(self):
        data=self.fixture();data[5][110]['digest']='sha256:'+'f'*64
        with self.assertRaisesRegex(ValueError,'digest differs'):self.verify(data)

    def test_main_evidence_for_another_commit_is_not_authorized_by_equal_tree(self):
        data=self.fixture();data[0]['sha']='f'*40
        with self.assertRaisesRegex(ValueError,'default-branch checkout'):self.verify(data)


class MeasurementContracts(unittest.TestCase):
    def fixture(self, os='linux', arch='x64', attempt=2):
        sha='a'*40;tree='b'*40;head='c'*40;base='d'*40
        abi,runner,platform={'linux':('linux-x64-gnu','ubuntu-24.04','Primary Linux'),'darwin':('darwin-arm64','macos-latest','Primary macOS'),'win32':('win32-x64-msvc','windows-latest','Primary Windows')}[os]
        policy=Path(__file__).with_name('performance-cohorts.mjs').read_bytes()
        plugins=module.ast.literal_eval(module.re.search(r'export const nativePlugins = (\[[^;]+\]);',policy.decode('utf-8'))[1])
        native={owner:{'sha256':'e'*64,'bytes':10} for owner in ['@farmfe/core',*['@farmfe/plugin-'+name for name in plugins]]}
        source={'sha':sha,'tree':tree,'files':{'scripts/performance-cohorts.mjs':policy}}
        run={'id':10,'run_attempt':2,'head_sha':head,'path':'.github/workflows/ci.yaml','event':'push','repository':{'id':5,'full_name':'farm-fe/farm'},'head_repository':{'id':5}}
        summary={'protocol':'commerce-business-2048-farm-supervised-v2'}
        row={'status':'complete','metadata':{'os':os,'arch':arch,'revision':sha,'baseSHA':base},'cells':[{'cacheMode':mode,'samples':3,'validSamples':3,'medianMs':100,'iqrMs':1} for mode in ['cold','warm']], 'comparison':{'mode':'current-only','baseCells':[]}}
        raw={**row,'metadata':{**row['metadata'],'artifactProducerSHA':sha,'harnessSha256':'f'*64},'protocol':summary['protocol'],'native':{'artifacts':native}}
        products={'abi':abi,'profile':'ci','originBuildSHA':sha,'runId':'10','runAttempt':'1','repository':'farm-fe/farm','records':[{'owner':owner,**value} for owner,value in native.items()]}
        starts={number:'2026-10-31T0'+str(number)+':00:00Z' for number in [1,2]}
        def job(number,name,origin):return {'id':number,'run_id':10,'name':name,'status':'completed','conclusion':'success','started_at':starts[origin],'completed_at':'2026-10-31T0'+str(origin)+':20:00Z'}
        measured=job(20,'Commerce Performance ('+runner+')',attempt)
        native_job=job(30,platform+' / native / Build and Upload User Artifacts - '+abi,1)
        jobs={1:([measured] if attempt==1 else [])+[native_job],2:([measured] if attempt==2 else [{**measured}])+[{**native_job}]}
        bodies={};artifacts={}
        def archive(number,name,value,member,origin):
            output=io.BytesIO()
            with zipfile.ZipFile(output,'w') as file:file.writestr(member,json.dumps(value))
            bodies[number]=output.getvalue();artifacts[number]={'id':number,'name':name,'expired':False,'created_at':'2026-10-31T0'+str(origin)+':10:00Z','workflow_run':{'id':10,'repository_id':5,'head_sha':head},'size_in_bytes':len(bodies[number]),'digest':'sha256:'+hashlib.sha256(bodies[number]).hexdigest()}
        archive(40,'performance-'+sha+'-'+abi+'-attempt-'+str(attempt),raw,'summary.json',attempt)
        archive(50,'native-shadow-'+sha+'-'+abi+'-1',products,'products.json',1)
        proof={'abi':abi,'profile':'ci','workflowPath':run['path'],'runId':10,'producerSHA':sha,'testedTree':tree,'runAttempt':attempt,'jobId':20,'artifactId':40,'artifactDigest':artifacts[40]['digest'],'rawSummarySha256':hashlib.sha256(json.dumps(raw).encode()).hexdigest(),'harnessSha256':'f'*64,'nativeAttempt':1,'nativeJobId':30,'nativeArtifactId':50,'nativeArtifactDigest':artifacts[50]['digest']}
        def get(route):
            if route.startswith('/actions/artifacts/'):return artifacts[int(route.split('/')[-1])]
            number=int(route.split('/attempts/')[1].split('/')[0])
            return {'jobs':jobs[number]} if '/jobs?' in route else {**run,'run_attempt':number,'run_started_at':starts[number]}
        transport=mock.Mock();transport.get.side_effect=get;transport.archive.side_effect=lambda number:bodies[number]
        return transport,run,source,summary,row,proof,jobs,artifacts,bodies,raw

    def verify(self,data):return module.verify_measurement_source(*data[:6])

    def test_actual_mixed_attempt_platform_sources_are_independently_verified(self):
        for os,arch,attempt in [('linux','x64',2),('darwin','arm64',1),('win32','x64',1)]:self.verify(self.fixture(os,arch,attempt))

    def test_complete_summary_verifies_all_original_platform_archives(self):
        cases=[self.fixture('linux','x64',2),self.fixture('darwin','arm64',1),self.fixture('win32','x64',1)]
        run=cases[0][1];source=cases[0][2];bodies={};artifacts={};jobs={1:[],2:[]};reports=[];proofs=[]
        for index,data in enumerate(cases):
            for attempt in [1,2]: jobs[attempt].extend(data[6][attempt])
            reports.append(data[4]);proofs.append(data[5])
            for old,key in [(40,'artifactId'),(50,'nativeArtifactId')]:
                new=old+index*100;data[5][key]=new;artifact={**data[7][old],'id':new};artifacts[new]=artifact;bodies[new]=data[8][old]
        summary={'schemaVersion':3,'protocol':'commerce-business-2048-farm-supervised-v2','runId':10,'runAttempt':2,'repositoryId':5,'event':'push','headSHA':run['head_sha'],'baseSHA':'d'*40,'prNumber':None,'reports':reports,'measurementSources':proofs}
        for row in reports:row['metadata'].update(fixtureSha256='e'*64,lockSha256='f'*64)
        # Freeze the real raw bytes again after adding the metadata being checked.
        for data,proof in zip(cases,proofs):
            raw=data[9];raw['metadata'].update(fixtureSha256='e'*64,lockSha256='f'*64)
            output=io.BytesIO()
            with zipfile.ZipFile(output,'w') as file:file.writestr('summary.json',json.dumps(raw))
            number=proof['artifactId'];bodies[number]=output.getvalue();artifacts[number].update(size_in_bytes=len(bodies[number]),digest='sha256:'+hashlib.sha256(bodies[number]).hexdigest());proof.update(artifactDigest=artifacts[number]['digest'],rawSummarySha256=hashlib.sha256(json.dumps(raw).encode()).hexdigest())
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as file:file.writestr('summary.json',json.dumps(summary))
        bodies[900]=output.getvalue();artifacts[900]={'id':900,'name':'performance-pr-summary-2','expired':False,'created_at':'2026-10-31T02:10:00Z','workflow_run':{'id':10},'size_in_bytes':len(bodies[900]),'digest':'sha256:'+hashlib.sha256(bodies[900]).hexdigest()}
        def get(route):
            if '/artifacts?' in route:return {'total_count':1,'artifacts':[artifacts[900]]}
            if route.startswith('/actions/artifacts/'):return artifacts[int(route.split('/')[-1])]
            attempt=int(route.split('/attempts/')[1].split('/')[0])
            return {'jobs':jobs[attempt]} if '/jobs?' in route else {**run,'run_attempt':attempt,'run_started_at':'2026-10-31T0'+str(attempt)+':00:00Z'}
        transport=mock.Mock();transport.get.side_effect=get;transport.archive.side_effect=lambda number:bodies[number]
        aggregation=[{'id':9000,'run_id':10,'name':'Aggregate informational performance report','status':'completed','conclusion':'success','started_at':'2026-10-31T02:00:00Z','completed_at':'2026-10-31T02:20:00Z'}]
        self.assertEqual(module.verify_performance(transport,run,aggregation,source,{'baseSHA':'d'*40})['measurementSources'][1]['runAttempt'],1)

    def test_attempt_start_time_is_not_used_as_a_carried_execution_identity(self):
        data=self.fixture('darwin','arm64',1);original=data[0].get.side_effect
        def get(route):
            value=original(route)
            return {**value,'run_started_at':'2026-10-31T01:00:00Z'} if '/attempts/' in route and '/jobs?' not in route else value
        data[0].get.side_effect=get;self.verify(data)

    def test_wrong_producer_digest_tree_profile_or_compacted_cells_reject(self):
        for mutate in [lambda d:d[5].update(testedTree='9'*40),lambda d:d[5].update(profile='release'),lambda d:d[5].update(jobId=99),lambda d:d[7][40].update(digest='sha256:'+'0'*64),lambda d:d[4].update(cells=[])]:
            data=self.fixture();mutate(data)
            with self.assertRaises(ValueError):self.verify(data)

    def test_later_failure_cannot_be_hidden_by_an_earlier_success(self):
        data=self.fixture('darwin','arm64',1)
        data[6][2][0].update(started_at='2026-10-31T02:01:00Z',completed_at='2026-10-31T02:02:00Z',conclusion='failure')
        with self.assertRaisesRegex(ValueError,'Latest actual|newer actual'):self.verify(data)

    def test_real_bytes_policy_requires_strict_utf8_and_declared_plugins(self):
        self.verify(self.fixture())
        for policy in [b'\xff', b'export const unrelated = [];']:
            data=self.fixture();data[2]['files']['scripts/performance-cohorts.mjs']=policy
            with self.assertRaises((ValueError,UnicodeDecodeError)):self.verify(data)

    def test_null_started_new_jobs_and_same_time_different_id_are_not_carried(self):
        for status,conclusion in [('completed','skipped'),('completed','cancelled'),('queued',None),('in_progress',None)]:
            data=self.fixture('darwin','arm64',1)
            data[6][2][0].update(status=status,conclusion=conclusion,started_at=None,completed_at=None)
            with self.assertRaisesRegex(ValueError,'Latest actual measurement producer'):self.verify(data)
        data=self.fixture('darwin','arm64',1);data[6][2][0]['id']=999
        with self.assertRaisesRegex(ValueError,'Unproven carried'):self.verify(data)

    def test_replaced_archive_after_metadata_lookup_rejects(self):
        data=self.fixture();data[8][40]+=b'changed'
        with self.assertRaisesRegex(ValueError,'archive bytes'):self.verify(data)


if __name__=='__main__':unittest.main(verbosity=2)
