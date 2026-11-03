"""Offline original-attempt proof fixtures; no network or publication."""
# cspell:words importlib urllib unstarted
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace
import urllib.parse
import zipfile

spec=importlib.util.spec_from_file_location('unit_shadow',Path(__file__).with_name('ci-e2e-shadow-inputs.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


def fixture(observed=None):
    identity={'headSHA':'a'*40,'baseSHA':'b'*40,'testedSHA':'c'*40,'testedTree':'d'*40,'runId':'41449476925','repository':'farm-fe/farm','policy':module.impact.POLICY}
    if observed:
        identity.update(testedSHA=observed['testedSHA'],runId=str(observed['runId']))
    run={'id':int(identity['runId']),'run_attempt':2,'path':'.github/workflows/ci.yaml','event':'pull_request','head_sha':identity['headSHA'],'repository':{'id':1,'full_name':'farm-fe/farm'},'head_repository':{'id':1}}
    plan={**identity,'runAttempt':'1','mode':'shadow','skipAuthorized':False,'reuseAuthorized':False,'executionCells':module.impact.execution_cells('e2e')}
    plan['planHash']=module.impact.digest(plan)
    jobs={1:[],2:[]};items={};bodies={};calls=[]
    def add(number,name,attempt,value,member,step,job_name):
        body=io.BytesIO()
        with zipfile.ZipFile(body,'w') as archive:archive.writestr(member,json.dumps(value))
        raw=body.getvalue();bodies[number]=raw
        items[name]={'id':number,'name':name,'expired':False,'workflow_run':{'id':run['id']},'size_in_bytes':len(raw),'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'created_at':f'2026-11-02T0{attempt}:02:00Z'}
        job={'id':number+100,'run_id':run['id'],'name':job_name,'status':'completed','conclusion':'success','started_at':f'2026-11-02T0{attempt}:00:00Z','completed_at':f'2026-11-02T0{attempt}:04:00Z','steps':[{'name':step,'status':'completed','conclusion':'success','started_at':f'2026-11-02T0{attempt}:01:00Z','completed_at':f'2026-11-02T0{attempt}:03:00Z'}]}
        jobs[attempt].append(job)
        if attempt==1:jobs[2].append(copy.deepcopy(job))
    add(1,'impact-plan-'+identity['testedSHA']+'-1',1,plan,'plan.json','Upload fixed source impact plan','Incremental CI plan (shadow only)')
    for i,(cell,name,key,suffix) in enumerate(module.CELLS):
        attempt=2 if i % 2 else 1
        if observed:
            expected='impact-cell-'+identity['testedSHA']+'-'+key+'-'+suffix+'-2'
            attempt=2 if expected in {row['name'] for row in observed['observedReceipts']} else 1
        value={**identity,'runAttempt':str(attempt),'cell':cell,'jobKey':key,'workflowRef':'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge','schemaVersion':2,'planBound':False,'status':'executed-success','acceptanceInherited':False,'authoritativeJobIdentityVerified':False}
        value['observationHash']=module.impact.digest(value)
        add(i+2,'impact-cell-'+identity['testedSHA']+'-'+key+'-'+suffix+'-'+str(attempt),attempt,value,'receipt.json','Preserve actual cell receipt (shadow only)',name)
    class API:
        def get(self,route):
            calls.append(route)
            if '/attempts/' in route:
                attempt=int(route.split('/attempts/')[1].split('/')[0])
                return {'jobs':jobs[attempt]} if '/jobs?' in route else {**run,'run_attempt':attempt}
            if '/artifacts?' in route:
                name=urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)['name'][0]
                return {'total_count':int(name in items),'artifacts':[items[name]] if name in items else []}
            return next(row for row in items.values() if row['id']==int(route.rsplit('/',1)[1]))
        def archive(self,number):return bodies[number]
    return API(),run,identity,jobs,items,bodies,calls


class Contracts(unittest.TestCase):
    def test_observed_attempt_two_names_and_inventory_replay(self):
        data=json.loads(Path(__file__).with_name('fixtures').joinpath('e2e-shadow-mixed-attempt-observation.json').read_text())
        self.assertEqual(len(data['observedReceipts']),23)
        self.assertEqual(data['requestedPlan'],'impact-plan-'+data['testedSHA']+'-2')
        api,run,identity,jobs,items,bodies,calls=fixture(data)
        plan,rows,bindings=module.resolve(api,run,identity)
        actual={name for name in items if name.startswith('impact-cell-') and name.endswith('-2')}
        self.assertEqual(actual,{row['name'] for row in data['observedReceipts']})
        for _,producer,_,_ in module.CELLS:
            matches=[job for job in data['jobs'] if job['name']==producer or job['name'].endswith(' / '+producer)]
            # First page omits later aggregate/performance jobs; no invented evidence.
            if matches:self.assertEqual(len(matches),1)
        self.assertEqual(bindings['plannerAttempt'],1)
        self.assertTrue(data['limitations'])

    def test_summary_real_resolver_preserves_original_attempts(self):
        spec=importlib.util.spec_from_file_location('e2e_summary_test',Path(__file__).with_name('ci-impact-summary.py'))
        summary=importlib.util.module_from_spec(spec);spec.loader.exec_module(summary)
        api,run,identity,*_=fixture();resolved=module.resolve(api,run,identity)
        for fails in [False,True]:
            error=ValueError('receipt unavailable');error.shadow_plan=resolved[0]
            resolver=SimpleNamespace(current=mock.Mock(side_effect=error) if fails else mock.Mock(return_value=resolved))
            loader=SimpleNamespace(exec_module=lambda _:None)
            with tempfile.TemporaryDirectory() as tmp:
                target=Path(tmp)/'report.json'
                keys=['impact-plan','release-contract-tests','performance-contract-tests','call-rust-build-user','call-rust-build-release','check-native-candidate-assembly','examples-test','docs-acceptance','type-check','ts-test','performance-report','performance-summary']
                env={'CI_E2E_RESOLVE_ORIGINAL_ATTEMPTS':'true','CI_IMPACT_SCOPE':'e2e','CI_IMPACT_NEEDS':json.dumps({key:{'result':'success'} for key in keys})}
                with mock.patch.dict(os.environ,env),mock.patch.object(sys,'argv',['summary','--plan','unused','--output',str(target)]),mock.patch.object(summary.importlib.util,'spec_from_file_location',return_value=SimpleNamespace(loader=loader)),mock.patch.object(summary.importlib.util,'module_from_spec',return_value=resolver):
                    if fails:
                        with self.assertRaises(SystemExit):summary.main()
                    else:summary.main()
                report=json.loads(target.read_text())
                self.assertFalse(report['publicationAuthorized'])
                self.assertEqual(report['cellBindingsVerified'],not fails)
                if not fails:
                    self.assertEqual(report['originalProducerBindings']['plannerAttempt'],1)
                    self.assertEqual(report['originalProducerBindings']['consumerAttempt'],2)

    def test_complete_mixed_attempt_inventory(self):
        api,run,identity,jobs,items,bodies,calls=fixture()
        plan,rows,bindings=module.resolve(api,run,identity)
        self.assertEqual(len(rows),27)
        self.assertEqual(plan['runAttempt'],'1')
        self.assertEqual(bindings['plannerAttempt'],1)
        self.assertEqual({row['runAttempt'] for row in rows},{'1','2'})
        self.assertEqual(sorted(row['cell'] for row in rows),sorted(module.impact.execution_cells('e2e')))
        self.assertFalse(any('impact-plan-' in call and call.endswith('-2') for call in calls))

    def test_latest_failure_cancel_skip_unstarted_and_carried_id_reject(self):
        for defect in ['failure','cancelled','skipped','unstarted','carried-id']:
            api,run,identity,jobs,*_=fixture()
            if defect=='carried-id':jobs[2][0]['id']+=1000
            elif defect=='unstarted':jobs[2][1]['started_at']=None
            else:jobs[2][1]['conclusion']=defect
            with self.assertRaises(ValueError):module.resolve(api,run,identity)

    def test_missing_each_artifact_rejects(self):
        for index in range(28):
            api,run,identity,jobs,items,*_=fixture()
            del items[list(items)[index]]
            with self.assertRaises(ValueError) as error:module.resolve(api,run,identity)
            self.assertEqual(hasattr(error.exception,'shadow_plan'),index!=0)

    def test_identity_digest_upload_and_source_reject(self):
        for defect in ['source','tree','attempt','workflow','digest','upload','run']:
            api,run,identity,jobs,items,bodies,_=fixture()
            if defect=='upload':
                jobs[1][1]['steps'][0]['conclusion']='failure'
                jobs[2][1]['steps'][0]['conclusion']='failure'
            elif defect=='run':run['head_repository']['id']=2
            else:
                artifact=list(items.values())[1];number=artifact['id']
                with zipfile.ZipFile(io.BytesIO(bodies[number])) as archive:row=json.loads(archive.read('receipt.json'))
                field={'source':'headSHA','tree':'testedTree','attempt':'runAttempt','workflow':'workflowRef','digest':'observationHash'}[defect]
                row[field]='wrong'
                if defect!='digest':row['observationHash']=module.impact.digest({k:v for k,v in row.items() if k!='observationHash'})
                out=io.BytesIO()
                with zipfile.ZipFile(out,'w') as archive:archive.writestr('receipt.json',json.dumps(row))
                bodies[number]=out.getvalue();artifact['size_in_bytes']=len(bodies[number]);artifact['digest']='sha256:'+hashlib.sha256(bodies[number]).hexdigest()
            with self.assertRaises(ValueError):module.resolve(api,run,identity)

if __name__=='__main__':unittest.main()
