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
        for attempt in jobs:jobs[attempt].append(copy.deepcopy(job))
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
        ledger=fixtures.producer.create_ledger(source,coverage,{'GITHUB_ACTIONS':'true','GITHUB_JOB':'performance-summary','GITHUB_REPOSITORY_ID':'1','GITHUB_RUN_ID':'2','GITHUB_RUN_ATTEMPT':'3','GITHUB_EVENT_NAME':'push'})
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
        job={'id':998,'run_id':2,'name':'Type Check & Changesets','status':'completed','conclusion':'success','started_at':'2026-10-31T01:00:00Z','completed_at':'2026-10-31T01:04:00Z','steps':[{'name':'Upload bound current suite selection','status':'completed','conclusion':'success','started_at':'2026-10-31T01:01:00Z','completed_at':'2026-10-31T01:03:00Z'}]}
        for attempt in jobs:
            jobs[attempt]=[item for item in jobs[attempt] if item['name']!=job['name']]+[copy.deepcopy(job)]
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'selection/selection.json'
            module.collect(transport,run,source,Path(tmp)/'receipts',hashlib.sha256(member).hexdigest(),target)
            self.assertEqual(target.read_bytes(),member)
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

if __name__=='__main__':unittest.main(verbosity=2)
