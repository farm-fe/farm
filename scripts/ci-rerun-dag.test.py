"""Offline same-source run3 DAG: planner1, suite1/2/3, measurements2/1/1, summary3."""
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


def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

functional=load('dag_functional_fixture','incremental-release-functional.test.py')
suite=load('dag_suite_fixture','ci-suite-attempts.test.py')

class Contracts(unittest.TestCase):
    def test_full_fixed_source_rerun_dag_keeps_all_original_attempts_and_names(self):
        source,bindings,old,base_jobs,runs,base_artifacts=functional.Contracts().fixture()
        old_bodies={number:old.archive(number) for number in base_artifacts}
        source['files']={'scripts/performance-cohorts.mjs':Path(__file__).with_name('performance-cohorts.mjs').read_bytes()}
        for scope in ['ci','unit']:bindings[scope]['attempt']=3;runs[bindings[scope]['runId']]['run_attempt']=3
        run=runs[10];run['repository']['full_name']='farm-fe/farm'
        metadata={};bodies={};jobs={number:copy.deepcopy(base_jobs[10]) for number in [1,2,3]}
        for number in [10,20]:
            plan=json.loads(zipfile.ZipFile(io.BytesIO(old_bodies[number])).read('plan.json'));plan['baseSHA']='d'*40
            plan['planHash']=functional.module.entry.chain.digest({key:value for key,value in plan.items() if key!='planHash'})
            body=io.BytesIO()
            with zipfile.ZipFile(body,'w') as archive:archive.writestr('plan.json',json.dumps(plan))
            bodies[number]=body.getvalue();metadata[number]={**base_artifacts[number],'size_in_bytes':len(bodies[number]),'digest':'sha256:'+hashlib.sha256(bodies[number]).hexdigest()}
        def store(identity,name,member,data,attempt):
            body=io.BytesIO()
            with zipfile.ZipFile(body,'w') as archive:archive.writestr(member,json.dumps(data))
            bodies[identity]=body.getvalue();metadata[identity]={'id':identity,'name':name,'expired':False,'workflow_run':{'id':10,'repository_id':5,'head_sha':source['sha']},'size_in_bytes':len(bodies[identity]),'digest':'sha256:'+hashlib.sha256(bodies[identity]).hexdigest(),'created_at':f'2026-10-31T0{attempt}:10:00Z'}
        def window(job,attempt):
            job['started_at']=f'2026-10-31T0{attempt}:00:00Z';job['completed_at']=f'2026-10-31T0{attempt}:20:00Z';return job
        # Only the six leaf suite jobs execute; the Examples wrapper rerun itself
        # does not manufacture six new receipts in attempt3.
        _,_,_,suite_jobs,suite_artifacts,suite_bodies,_=suite.fixture()
        for index,(name,artifact) in enumerate(suite_artifacts.items()):
            member='ts.json' if '-suite-ts-' in name else 'browser.json'
            data=json.loads(zipfile.ZipFile(io.BytesIO(suite_bodies[artifact['id']])).read(member))
            attempt=[2,1,1,1,1,3][index];data['run'].update(repositoryId=5,runId=10,runAttempt=attempt)
            new_name=name[:-1]+str(attempt);store(3000+index,new_name,member,data,attempt)
            for number in jobs:
                jobs[number]=[job for job in jobs[number] if not job['name'].startswith(('Examples Test (','Typescript Test ('))]
            original=next(job for job in suite_jobs[1] if job['name'].startswith('Primary ') and (('Typescript' in job['name']) == (member=='ts.json')) and suite.module.execution.PLATFORMS[data['abi']] in job['name'])
            original=window({**copy.deepcopy(original),'run_id':10},attempt)
            original['steps'][0].update(started_at=original['started_at'],completed_at=original['completed_at'])
            for number in range(attempt,4):jobs[number].append(copy.deepcopy(original))
        reports=[];proofs=[]
        for index,(os,arch,attempt) in enumerate([('linux','x64',2),('darwin','arm64',1),('win32','x64',1)]):
            data=functional.MeasurementContracts().fixture(os,arch,attempt);row,proof=data[4],data[5]
            row['metadata'].update(fixtureSha256='e'*64,lockSha256='f'*64);data[9]['metadata'].update(fixtureSha256='e'*64,lockSha256='f'*64)
            for old_id,key,member in [(40,'artifactId','summary.json'),(50,'nativeArtifactId','products.json')]:
                identity=1000+index*100+old_id;old_meta=data[7][old_id]
                value=json.loads(zipfile.ZipFile(io.BytesIO(data[8][old_id])).read(member))
                if old_id==40:value=data[9]
                store(identity,old_meta['name'],member,value,attempt if old_id==40 else 1);proof[key]=identity
                proof['artifactDigest' if old_id==40 else 'nativeArtifactDigest']=metadata[identity]['digest']
                if old_id==40:proof['rawSummarySha256']=hashlib.sha256(json.dumps(value).encode()).hexdigest()
            proof['jobId']+=index*100;proof['nativeJobId']+=index*100
            for number in jobs:
                jobs[number]=[job for job in jobs[number] if job['name']!='Commerce Performance ('+functional.module.PLATFORMS[index]+')']
                for job in data[6][min(number,2)]:
                    job=copy.deepcopy(job);job['id']+=index*100;job['run_id']=10;jobs[number].append(job)
            reports.append(row);proofs.append(proof)
        summary={'schemaVersion':3,'protocol':'commerce-business-2048-farm-supervised-v2','repositoryId':5,'runId':10,'runAttempt':3,'event':'push','headSHA':source['sha'],'baseSHA':'d'*40,'prNumber':None,'reports':reports,'measurementSources':proofs}
        store(9999,'performance-pr-summary-3','summary.json',summary,3)
        for number in jobs:
            if number < 3:jobs[number]=[job for job in jobs[number] if job['name']!='Aggregate informational performance report']
            for job in jobs[number]:
                if job['name']=='Aggregate informational performance report':window(job,3)
        class API:
            def get(self,route):
                if route=='':return {'id':5,'full_name':'farm-fe/farm','default_branch':'main'}
                if route.startswith('/git/commits/'):return {'tree':{'sha':source['tree']}}
                if '/artifacts?' in route:
                    name=urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)['name'][0];run_id=int(route.split('/')[3]);found=[row for row in metadata.values() if row['name']==name and row['workflow_run']['id']==run_id];return {'total_count':len(found),'artifacts':found}
                if route.startswith('/actions/artifacts/'):return metadata[int(route.rsplit('/',1)[1])]
                run_id=int(route.split('/')[3]);attempt=int(route.split('/attempts/')[1].split('/')[0])
                if '/jobs?' in route:return {'jobs':jobs[attempt] if run_id==10 else base_jobs[run_id]}
                return {**runs[run_id],'run_attempt':attempt}
            def archive(self,identity):return bodies[identity]
        api=API()
        with tempfile.TemporaryDirectory() as tmp:
            resolved=suite.module.collect(api,run,source,Path(tmp)/'suites')
            self.assertEqual({row['runAttempt'] for row in resolved.values()},{1,2,3})
        result=functional.module.verify_functional(api,source,bindings)
        self.assertTrue(result['functionalAcceptanceVerified']);self.assertFalse(result['buildBytesReused'])
        self.assertEqual([row['runAttempt'] for row in summary['measurementSources']],[2,1,1])

if __name__=='__main__':unittest.main(verbosity=2)
