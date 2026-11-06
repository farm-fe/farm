# cspell:words unstarted
"""Offline original ZIP/job fixtures for rerun subgraphs. No GitHub access or build."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import urllib.parse
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file)); value = importlib.util.module_from_spec(spec); spec.loader.exec_module(value); return value

module = load('suite_attempt_contract', 'ci-suite-attempts.py')
fixtures = load('suite_attempt_fixture', 'ci-execution-evidence.test.py')


def fixture():
    args, discovery = fixtures.fixture()
    old, run, repository, ledger, ledger_meta, current_jobs, source, trusted, required = args
    run['run_attempt'] = 3
    jobs = {1:[], 2:[], 3:[copy.deepcopy(job) for job in current_jobs if not job['name'].startswith(('Examples Test (','Typescript Test ('))]}
    artifacts, bodies = {}, {}
    for index, cell in enumerate(ledger['coverage']['cells']):
        abi, kind = cell['abi'], cell['kind']
        label = 'ts' if kind == 'ts' else 'browser'
        prefix = {'linux-x64-gnu':'Primary Linux','darwin-arm64':'Primary macOS','win32-x64-msvc':'Primary Windows'}[abi]
        job_label = 'Typescript Test' if kind == 'ts' else 'Examples Test'
        step_label = 'Preserve actual TypeScript discovery and execution counts' if kind == 'ts' else 'Preserve actual browser discovery and execution counts'
        data = {key:copy.deepcopy(value) for key,value in cell.items() if key != 'artifactName'}
        data.update(chain='ci', acceptanceInherited=False, schemaVersion=1, receiptKind='actual-suite-execution-not-byte-reuse')
        name = cell['artifactName']; body = io.BytesIO()
        with zipfile.ZipFile(body,'w') as archive:archive.writestr(label + '.json',json.dumps(data))
        raw=body.getvalue();identity=100+index
        artifacts[name]={'id':identity,'name':name,'expired':False,'workflow_run':{'id':run['id']},'size_in_bytes':len(raw),'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'created_at':'2026-10-31T01:02:00Z'};bodies[identity]=raw
        job={'id':200+index,'run_id':run['id'],'name':prefix+' / '+job_label+' ('+module.execution.PLATFORMS[abi]+')','status':'completed','conclusion':'success','started_at':'2026-10-31T01:00:00Z','completed_at':'2026-10-31T01:04:00Z','steps':[{'name':step_label,'status':'completed','conclusion':'success','started_at':'2026-10-31T01:01:00Z','completed_at':'2026-10-31T01:03:00Z'}]}
        for attempt in jobs:jobs[attempt].append({**copy.deepcopy(job), 'head_sha':run['head_sha'], 'run_attempt':attempt})
    class Transport:
        calls=[]
        def get(self,route):
            self.calls.append(route)
            if '/attempts/' in route:
                attempt=int(route.split('/attempts/')[1].split('/')[0])
                return {'jobs':jobs[attempt]} if '/jobs?' in route else {**run,'run_attempt':attempt}
            if '/artifacts?' in route:
                name=urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)['name'][0]
                return {'artifacts':[artifacts[name]] if name in artifacts else [],'total_count':int(name in artifacts)}
            return next(item for item in artifacts.values() if item['id']==int(route.rsplit('/',1)[1]))
        def archive(self,identity):return bodies[identity]
    return Transport(),run,source,jobs,artifacts,bodies,discovery


class Contracts(unittest.TestCase):
    def test_carried_failure_diagnostic_distinguishes_identity_status_and_conclusion(self):
        for defect, expected in [('id', 'job-id'), ('status', 'prior-status'), ('conclusion', 'prior-conclusion')]:
            transport,run,source,jobs,*_=fixture()
            name='Primary Linux / Typescript Test (ubuntu-24.04)'
            current=next(row for row in jobs[3] if row['name']==name)
            prior=next(row for row in jobs[2] if row['name']==name)
            if defect=='id':current['id']=999
            elif defect=='status':prior['status']='in_progress'
            else:prior['conclusion']='failure'
            with self.assertRaises(ValueError) as error:
                module.execution.suite_receipt(transport,run,source,'ts','linux-x64-gnu')
            message=str(error.exception)
            self.assertTrue(message.startswith('Unproven carried suite producer: '))
            detail=json.loads(message.split(': ',1)[1])
            self.assertEqual(detail['mismatches'],[expected])
            self.assertEqual(detail['runId'],run['id'])
            self.assertEqual(detail['currentAttempt'],3)
            self.assertEqual(detail['priorAttempt'],2)
            self.assertEqual(detail['current']['id'],current['id'])
            self.assertEqual(detail['prior']['id'],prior['id'])
            self.assertEqual(detail['current']['started_at'],current['started_at'])
            self.assertEqual(detail['current']['completed_at'],current['completed_at'])

    def test_carried_diagnostic_is_bounded_and_does_not_log_arbitrary_metadata(self):
        transport,run,source,jobs,*_=fixture()
        name='Primary Linux / Typescript Test (ubuntu-24.04)'
        current=next(row for row in jobs[3] if row['name']==name)
        prior=next(row for row in jobs[2] if row['name']==name)
        current.update(id=999, secret='do-not-log', runner_name='do-not-log', run_attempt='do-not-log')
        prior.update(status='do-not-log', conclusion='do-not-log', steps=[{'name':'do-not-log'*10000}])
        for row in [current,prior]:row.update(started_at='do-not-log'*10000,completed_at='do-not-log'*10000)
        with self.assertRaises(ValueError) as error:
            module.execution.suite_receipt(transport,run,source,'ts','linux-x64-gnu')
        message=str(error.exception)
        self.assertNotIn('do-not-log',message)
        self.assertLess(len(message),2048)
        detail=json.loads(message.split(': ',1)[1])
        self.assertEqual(detail['mismatches'],['job-id','prior-status','prior-conclusion'])
        self.assertIsNone(detail['current']['run_attempt'])
        self.assertIsNone(detail['current']['started_at'])
        self.assertIsNone(detail['prior']['completed_at'])
        self.assertEqual(detail['prior']['status'],'invalid')
        self.assertEqual(set(detail['current']),{'id','run_id','run_attempt','status','conclusion','started_at','completed_at'})

    def test_attempt3_aggregate_only_resolves_six_original_attempt1_receipts(self):
        transport,run,source,jobs,artifacts,bodies,_=fixture()
        with tempfile.TemporaryDirectory() as tmp:
            proofs=module.collect(transport,run,source,Path(tmp)/'receipts')
            self.assertEqual(len(proofs),6)
            self.assertEqual({value['runAttempt'] for value in proofs.values()},{1})
            self.assertTrue(all(name.endswith('-1') for name in proofs))
            index=json.loads((Path(tmp)/'receipts/producers.json').read_text())
            self.assertEqual(index['aggregationAttempt'],3)
            self.assertTrue(all('/attempts/3' not in name for name in artifacts))

    def test_baseline_independently_replays_same_original_producers(self):
        transport,run,source,jobs,artifacts,bodies,discovery=fixture()
        cells=[]
        for abi in module.execution.PLATFORMS:
            for kind in ['ts','examples-browser']:
                data,artifact,binding=module.execution.suite_receipt(transport,run,source,kind,abi)
                cells.append({**data,'artifactName':artifact['name'],'producerBinding':binding})
        coverage={'kind':'complete-ci-suite-counts-not-build-bytes','sourceSHA':source['sha'],'sourceTree':source['tree'],
                  'run':{'repositoryId':1,'runId':2,'runAttempt':3,'event':'push'},'cells':cells}
        ledger=fixtures.producer.create_ledger(source,coverage,{'GITHUB_ACTIONS':'true','GITHUB_JOB':'execution-baseline','GITHUB_REPOSITORY_ID':'1','GITHUB_RUN_ID':'2','GITHUB_RUN_ATTEMPT':'3','GITHUB_EVENT_NAME':'push'})
        meta={'name':'farm-ci-execution-baseline-2-3','workflow_run':{'id':2},'expired':False,'id':3,'digest':'sha256:'+'0'*64,'created_at':'2026-10-31T01:02:00Z'}
        result=module.execution.verify_execution_ledger(transport,run,{'id':1,'default_branch':'main'},ledger,meta,jobs[3],source,source,ledger['requiredJobs'],suite_discovery=lambda *_:discovery)
        self.assertFalse(result['buildBytesReused'])
        ledger['coverage']['cells'][0]['producerBinding']['runAttempt']=3
        with self.assertRaisesRegex(ValueError,'producer proof differs'):
            module.execution.verify_execution_ledger(transport,run,{'id':1,'default_branch':'main'},ledger,meta,jobs[3],source,source,ledger['requiredJobs'],suite_discovery=lambda *_:discovery)

    def test_selection_is_restored_from_its_actual_typecheck_attempt_and_exact_bytes(self):
        transport,run,source,jobs,artifacts,bodies,_=fixture()
        value={'sourceSHA':source['sha'],'sourceTree':source['tree'],'runId':run['id'],'attempt':1,'repositoryId':run['repository']['id']}
        member=json.dumps(value).encode();body=io.BytesIO()
        with zipfile.ZipFile(body,'w') as archive:archive.writestr('selection.json',member)
        raw=body.getvalue();name='ci-suite-selection-2-1';artifacts[name]={'id':999,'name':name,'expired':False,'workflow_run':{'id':2},'size_in_bytes':len(raw),'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'created_at':'2026-10-31T01:02:00Z'};bodies[999]=raw
        job={'id':998,'run_id':2,'name':'Type Check','status':'completed','conclusion':'success','started_at':'2026-10-31T01:00:00Z','completed_at':'2026-10-31T01:04:00Z','steps':[{'name':'Upload bound current suite selection','status':'completed','conclusion':'success','started_at':'2026-10-31T01:01:00Z','completed_at':'2026-10-31T01:03:00Z'}]}
        for attempt in jobs:
            jobs[attempt]=[item for item in jobs[attempt] if item['name']!=job['name']]+[copy.deepcopy(job)]
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'selection/selection.json'
            module.collect(transport,run,source,Path(tmp)/'receipts',hashlib.sha256(member).hexdigest(),target)
            self.assertEqual(target.read_bytes(),member)
            # A successful legacy compatibility context is not a selection producer.
            for attempt in jobs:
                next(item for item in jobs[attempt] if item['id']==998)['name']='Type Check & Changesets'
            with self.assertRaises(ValueError):
                module.original_selection(transport,run,source,hashlib.sha256(member).hexdigest())
            for attempt in jobs:
                next(item for item in jobs[attempt] if item['id']==998)['name']='Type Check'
            self.assertEqual(json.loads((Path(tmp)/'receipts/producers.json').read_text())['selectionAttempt'],1)
            with self.assertRaisesRegex(ValueError,'selection bytes differ'):
                module.collect(transport,run,source,Path(tmp)/'bad','0'*64,Path(tmp)/'bad-selection.json')

    def test_new_failed_skipped_or_unknown_suite_cannot_fall_back_to_old_success(self):
        for conclusion in ['failure','skipped',None]:
            transport,run,source,jobs,*_=fixture()
            job=copy.deepcopy(jobs[1][0]);job.update(started_at='2026-10-31T03:00:00Z',completed_at='2026-10-31T03:03:00Z',conclusion=conclusion)
            jobs[3]=[item for item in jobs[3] if item['name']!=job['name']]+[job]
            with self.assertRaisesRegex(ValueError,'Latest actual suite producer'):
                module.collect(transport,run,source,Path('never-created'))

    def test_unstarted_new_suite_and_same_time_different_id_fail_closed(self):
        for status,conclusion in [('completed','skipped'),('completed','cancelled'),('queued',None),('in_progress',None)]:
            transport,run,source,jobs,*_=fixture()
            job=next(row for row in jobs[3] if row['name']=='Primary Linux / Typescript Test (ubuntu-24.04)')
            job.update(status=status,conclusion=conclusion,started_at=None,completed_at=None)
            with self.assertRaisesRegex(ValueError,'Latest actual suite producer'):
                module.execution.suite_receipt(transport,run,source,'ts','linux-x64-gnu')
        transport,run,source,jobs,*_=fixture()
        next(row for row in jobs[3] if row['name']=='Primary Linux / Typescript Test (ubuntu-24.04)')['id']=999
        with self.assertRaisesRegex(ValueError,'Unproven carried suite producer'):
            module.execution.suite_receipt(transport,run,source,'ts','linux-x64-gnu')

    def test_wrong_source_profile_plan_bytes_or_upload_window_reject(self):
        for mutation in ['source','profile','digest','window']:
            transport,run,source,jobs,artifacts,bodies,_=fixture();artifact=next(iter(artifacts.values()))
            if mutation=='digest':artifact['digest']='sha256:'+'0'*64
            elif mutation=='window':artifact['created_at']='2026-10-31T03:02:00Z'
            else:
                with zipfile.ZipFile(io.BytesIO(bodies[artifact['id']])) as archive:data=json.loads(archive.read('ts.json'))
                data['sourceTree' if mutation=='source' else 'chain']='invalid'
                body=io.BytesIO()
                with zipfile.ZipFile(body,'w') as archive:archive.writestr('ts.json',json.dumps(data))
                raw=body.getvalue();bodies[artifact['id']]=raw;artifact.update(size_in_bytes=len(raw),digest='sha256:'+hashlib.sha256(raw).hexdigest())
            with self.assertRaises(ValueError):module.collect(transport,run,source,Path('never-created'))

    def test_intermediate_actual_suite_attempt_retains_its_original_name(self):
        transport,run,source,jobs,artifacts,bodies,_=fixture()
        name=next(iter(artifacts));artifact=artifacts.pop(name);newname=name[:-1]+'2';artifact.update(name=newname,created_at='2026-10-31T02:02:00Z');artifacts[newname]=artifact
        with zipfile.ZipFile(io.BytesIO(bodies[artifact['id']])) as archive:data=json.loads(archive.read('ts.json'))
        data['run']['runAttempt']=2;body=io.BytesIO()
        with zipfile.ZipFile(body,'w') as archive:archive.writestr('ts.json',json.dumps(data))
        raw=body.getvalue();bodies[artifact['id']]=raw;artifact.update(size_in_bytes=len(raw),digest='sha256:'+hashlib.sha256(raw).hexdigest())
        for attempt in [2,3]:
            next(item for item in jobs[attempt] if item['name']=='Primary Linux / Typescript Test (ubuntu-24.04)').update(started_at='2026-10-31T02:00:00Z',completed_at='2026-10-31T02:04:00Z',steps=[{'name':'Preserve actual TypeScript discovery and execution counts','status':'completed','conclusion':'success','started_at':'2026-10-31T02:01:00Z','completed_at':'2026-10-31T02:03:00Z'}])
        with tempfile.TemporaryDirectory() as tmp:
            proofs=module.collect(transport,run,source,Path(tmp)/'receipts');self.assertEqual(proofs[newname]['runAttempt'],2)
            self.assertEqual(sum(value['runAttempt']==1 for value in proofs.values()),5)

def enrich(run,jobs):
    for attempt,rows in jobs.items():
        for row in rows:
            row.update(head_sha=run['head_sha'],run_attempt=attempt,runner_id=44,runner_name='hosted-runner')
            for number,step in enumerate(row.get('steps',[]),1):step['number']=number
            row['id']+=10000*attempt


class ChangedViewContracts(unittest.TestCase):
    def test_changed_ids_resolve_original_suite_upload_and_bytes(self):
        api,run,source,jobs,*_=fixture();enrich(run,jobs)
        actual,artifact,binding=module.execution.suite_receipt(api,run,source,'ts','linux-x64-gnu')
        self.assertEqual(binding['runAttempt'],1)
        self.assertEqual(binding['jobId'],10200)
        self.assertEqual(binding['artifactDigest'],artifact['digest'])

    def test_observed_linux_attempt2_to3_identity_and_fractional_step_times(self):
        # IDs and job times come from run 41759140572 gate diagnostics.
        # Runner/step details below are local proof fixtures, not downloaded evidence.
        run={'id':41759140572,'head_sha':'b'*40}
        prior={'id':126406306733,'run_id':run['id'],'run_attempt':2,'head_sha':run['head_sha'],
               'name':'Primary Linux / Typescript Test (ubuntu-24.04)','runner_id':44,'runner_name':'hosted-runner',
               'status':'completed','conclusion':'success','started_at':'2026-11-04T05:25:03Z','completed_at':'2026-11-04T05:27:32Z',
               'steps':[{'number':1,'name':'Set up job','status':'completed','conclusion':'success',
                         'started_at':'2026-11-04T05:25:03.000Z','completed_at':'2026-11-04T05:25:04.000Z'}]}
        current={**copy.deepcopy(prior),'id':126425320863,'run_attempt':3,'runner_group_id':None}
        prior['runner_group_id']=0
        self.assertTrue(module.execution.same_carried_execution(current,prior,run,3))
        current.pop('runner_id')
        self.assertFalse(module.execution.same_carried_execution(current,prior,run,3))

    def test_changed_ids_in_unit_and_e2e_planners_and_cells(self):
        for file,scope in [('ci-unit-shadow-inputs.test.py','unit'),('ci-e2e-shadow-inputs.test.py','e2e')]:
            shadow_fixture=load('carried_'+scope,file)
            api,run,identity,jobs,*_=shadow_fixture.fixture();enrich(run,jobs)
            # E2E fixture exposes its scope-bound module wrapper.
            shadow_fixture.module.resolve(api,run,identity)

    def test_changed_id_missing_or_forged_execution_proof_rejects(self):
        for defect in ['source','attempt','runner','steps','step-time','step-status','step-number','name','failure']:
            with self.subTest(defect=defect):
                api,run,source,jobs,*_=fixture();enrich(run,jobs)
                row=next(x for x in jobs[3] if x['name']=='Primary Linux / Typescript Test (ubuntu-24.04)')
                if defect=='source':row['head_sha']='0'*40
                elif defect=='attempt':row['run_attempt']=1
                elif defect=='runner':row['runner_id']=45
                elif defect=='steps':row.pop('steps')
                elif defect=='step-time':row['steps'][0]['completed_at']='2026-10-31T01:02:59Z'
                elif defect=='step-status':row['steps'][0]['conclusion']='failure'
                elif defect=='step-number':row['steps'][0].pop('number')
                elif defect=='name':row['name']='evil / '+row['name']
                else:row['conclusion']='failure'
                with self.assertRaises(ValueError):module.execution.suite_receipt(api,run,source,'ts','linux-x64-gnu')


if __name__=='__main__':unittest.main(verbosity=2)
