"""Offline original-attempt proof fixtures; no network or publication."""
# cspell:words importlib urllib
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

spec=importlib.util.spec_from_file_location('unit_shadow',Path(__file__).with_name('ci-unit-shadow-inputs.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


def fixture():
    identity={'headSHA':'a'*40,'baseSHA':'b'*40,'testedSHA':'c'*40,'testedTree':'d'*40,'runId':'41449476925','repository':'farm-fe/farm','policy':module.impact.POLICY}
    run={'id':41449476925,'run_attempt':2,'path':'.github/workflows/rust-test.yaml','event':'pull_request','head_sha':identity['headSHA'],'repository':{'id':1,'full_name':'farm-fe/farm'},'head_repository':{'id':1}}
    plan={**identity,'runAttempt':'1','mode':'shadow','skipAuthorized':False,'reuseAuthorized':False,'executionCells':module.impact.execution_cells('unit')}
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
    add(1,'impact-plan-'+identity['testedSHA']+'-1',1,plan,'plan.json','Upload fixed source impact plan','Incremental Unit CI plan (shadow only)')
    for i,(cell,name,key,suffix) in enumerate(module.CELLS):
        attempt=2 if suffix=='ubuntu-24.04' else 1
        value={**identity,'runAttempt':str(attempt),'cell':cell,'jobKey':key,'workflowRef':'farm-fe/farm/.github/workflows/rust-test.yaml@refs/pull/2330/merge','schemaVersion':2,'planBound':False,'status':'executed-success','acceptanceInherited':False,'authoritativeJobIdentityVerified':False}
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
    def test_observed_run_replay_preserves_names_without_guessing_api_times(self):
        data=json.loads(Path(__file__).with_name('fixtures').joinpath('unit-shadow-mixed-attempt-observation.json').read_text())
        sha=data['testedSHA']
        self.assertEqual(data['planner']['artifactName'],'impact-plan-'+sha+'-1')
        self.assertEqual(data['summary']['requestedPlan'],'impact-plan-'+sha+'-2')
        self.assertTrue(data['linux']['artifactName'].endswith('-2'))
        self.assertTrue(data['check']['artifactName'].endswith('-1'))
        self.assertNotEqual(data['planner']['originalJobId'],data['planner']['latestListedJobId'])
        self.assertEqual(data['summary']['downloadedReceiptCount'],1)
        # The observed artifact names are replayed, while API times/ZIP bytes
        # remain synthetic fixture data and cannot authorize this historical run.
        api,run,identity,jobs,items,bodies,_=fixture()
        old=identity['testedSHA']; identity['testedSHA']=sha
        # Wrong actual checkout is rejected, not silently mapped to these names.
        with self.assertRaises(ValueError):module.resolve(api,run,identity)
        self.assertNotEqual(old,sha)
        self.assertTrue(data['limitations'])

    def test_resolution_reports_exact_stage_cell_and_original_ids(self):
        for defect,stage,reason in [('window','bind-upload','upload-time-window'),('missing','read-artifact','artifact-missing-or-ambiguous'),('bytes','read-artifact','artifact-bytes-differ')]:
            api,run,identity,jobs,items,bodies,_=fixture()
            name=next(name for name in items if 'cargo-check' in name)
            if defect=='window':items[name]['created_at']='2026-11-02T01:05:00Z'
            elif defect=='missing':del items[name]
            else:bodies[2]+=b'corrupt'
            with self.assertRaises(ValueError) as failure:module.resolve(api,run,identity)
            detail=failure.exception.shadow_context
            self.assertEqual(detail['resolutionStep'],stage)
            self.assertEqual(detail['resolutionReason'],reason)
            self.assertEqual(detail['cell'],'cargo-check')
            self.assertEqual(detail['producerAttempt'],1)
            self.assertEqual(detail['producerJobId'],102)
            if defect=='window':self.assertEqual(detail['artifactId'],2)
            self.assertEqual(failure.exception.shadow_plan['runAttempt'],'1')

    def test_mixed_attempts_keep_original_plan_and_per_cell_identity(self):
        api,run,identity,jobs,items,bodies,calls=fixture()
        plan,observations,bindings=module.resolve(api,run,identity)
        self.assertEqual(plan['runAttempt'],'1')
        self.assertEqual(bindings['consumerAttempt'],2)
        self.assertEqual(bindings['plannerAttempt'],1)
        self.assertEqual({row['cell']:row['runAttempt'] for row in observations},{cell:('2' if suffix=='ubuntu-24.04' else '1') for cell,_,_,suffix in module.CELLS})
        self.assertTrue(any('impact-plan-' in call and call.endswith('-1') for call in calls))
        self.assertFalse(any('impact-plan-' in call and call.endswith('-2') for call in calls))

    def test_carried_different_job_id_still_rejects_without_proof(self):
        api,run,identity,jobs,*_=fixture()
        jobs[2][0]['id']+=1000
        with self.assertRaisesRegex(ValueError,'Unproven carried suite producer'):module.resolve(api,run,identity)

    def test_latest_failed_producer_never_falls_back(self):
        for conclusion in ['failure','cancelled','skipped']:
            api,run,identity,jobs,*_=fixture()
            jobs[2][1]['conclusion']=conclusion
            with self.assertRaisesRegex(ValueError,'Latest actual suite producer'):module.resolve(api,run,identity)

    def test_missing_artifact_and_corrupt_archive_reject(self):
        for defect in ['missing','bytes']:
            api,run,identity,jobs,items,bodies,_=fixture()
            if defect=='missing':items.pop(next(name for name in items if 'cargo-check' in name))
            else:bodies[2]+=b'corrupt'
            with self.assertRaises(ValueError):module.resolve(api,run,identity)

    def test_self_consistent_invalid_zip_and_json_shape_report_observation_unavailable(self):
        for defect in ['zip','json-shape']:
            api,run,identity,jobs,items,bodies,_=fixture()
            artifact=next(row for name,row in items.items() if 'cargo-check' in name)
            if defect=='zip':raw=b'not a zip'
            else:
                target=io.BytesIO()
                with zipfile.ZipFile(target,'w') as archive:archive.writestr('receipt.json','[]')
                raw=target.getvalue()
            bodies[artifact['id']]=raw;artifact['size_in_bytes']=len(raw);artifact['digest']='sha256:'+hashlib.sha256(raw).hexdigest()
            with self.assertRaises(ValueError) as failure:module.resolve(api,run,identity)
            self.assertEqual(failure.exception.shadow_plan['runAttempt'],'1')

    def test_wrong_source_and_attempt_even_with_recomputed_digest_reject(self):
        for field,value in [('headSHA','e'*40),('testedSHA','e'*40),('runAttempt','2'),('cell','other')]:
            api,run,identity,jobs,items,bodies,_=fixture()
            artifact=next(row for name,row in items.items() if 'cargo-check' in name)
            number=artifact['id']
            with zipfile.ZipFile(io.BytesIO(bodies[number])) as archive:row=json.loads(archive.read('receipt.json'))
            row[field]=value;row['observationHash']=module.impact.digest({key:v for key,v in row.items() if key!='observationHash'})
            target=io.BytesIO()
            with zipfile.ZipFile(target,'w') as archive:archive.writestr('receipt.json',json.dumps(row))
            bodies[number]=target.getvalue();artifact['size_in_bytes']=len(bodies[number]);artifact['digest']='sha256:'+hashlib.sha256(bodies[number]).hexdigest()
            with self.assertRaises(ValueError):module.resolve(api,run,identity)

    def test_summary_preserves_mixed_attempts_and_writes_resolution_failure(self):
        spec=importlib.util.spec_from_file_location('unit_summary_test',Path(__file__).with_name('ci-impact-summary.py'))
        summary=importlib.util.module_from_spec(spec);spec.loader.exec_module(summary)
        api,run,identity,*_=fixture();resolved=module.resolve(api,run,identity)
        for fails in [False,True]:
            error=ValueError('receipt unavailable');error.shadow_plan=resolved[0]
            resolver=SimpleNamespace(current=mock.Mock(side_effect=error) if fails else mock.Mock(return_value=resolved))
            loader=SimpleNamespace(exec_module=lambda _:None)
            with tempfile.TemporaryDirectory() as tmp:
                target=Path(tmp)/'report.json'
                env={'CI_UNIT_RESOLVE_ORIGINAL_ATTEMPTS':'true','CI_IMPACT_SCOPE':'unit','CI_IMPACT_NEEDS':json.dumps({key:{'result':'success'} for key in ['impact-plan','cargo-check','cargo-test']})}
                with mock.patch.dict(os.environ,env), mock.patch.object(sys,'argv',['summary','--plan','unused','--output',str(target)]), mock.patch.object(summary.importlib.util,'spec_from_file_location',return_value=SimpleNamespace(loader=loader)), mock.patch.object(summary.importlib.util,'module_from_spec',return_value=resolver):
                    if fails:
                        with self.assertRaises(SystemExit):summary.main()
                    else:summary.main()
                report=json.loads(target.read_text())
                self.assertFalse(report['publicationAuthorized'])
                self.assertIsNone(report['planUnavailable'])
                if fails:
                    self.assertEqual(report['inputResolutionUnavailable']['stage'],'read-observations')
                    self.assertFalse(report['cellBindingsVerified'])
                else:
                    self.assertTrue(report['cellBindingsVerified'])
                    self.assertEqual(report['originalProducerBindings']['plannerAttempt'],1)
                    self.assertEqual(report['originalProducerBindings']['consumerAttempt'],2)

    def test_missing_plan_reports_unavailable_without_false_sha_mismatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            output=Path(tmp)/'report.json'
            env={**os.environ,'CI_IMPACT_SCOPE':'unit','CI_IMPACT_NEEDS':json.dumps({key:{'result':'success'} for key in ['impact-plan','cargo-check','cargo-test']})}
            env.pop('CI_UNIT_RESOLVE_ORIGINAL_ATTEMPTS',None)
            result=subprocess.run([sys.executable,'-B',str(Path(__file__).with_name('ci-impact-summary.py')),'--plan',str(Path(tmp)/'missing'),'--output',str(output)],env=env,capture_output=True,text=True)
            self.assertEqual(result.returncode,1)
            report=json.loads(output.read_text())
            self.assertEqual(report['planUnavailable']['stage'],'read-plan')
            self.assertFalse(report['cellBindingsVerified'])
            self.assertNotIn('cellBindingsError',report)
            self.assertNotIn('identity-mismatch',result.stderr)
            self.assertFalse(report['publicationAuthorized'])

if __name__=='__main__':unittest.main()
