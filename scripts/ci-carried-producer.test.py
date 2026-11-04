"""Changed Actions job IDs require complete original execution evidence."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest

def load(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result

suite=load('carried_suite','ci-suite-attempts.test.py')

def enrich(run,jobs):
    for attempt,rows in jobs.items():
        for row in rows:
            row.update(head_sha=run['head_sha'],run_attempt=attempt,runner_id=44,runner_name='hosted-runner')
            for number,step in enumerate(row.get('steps',[]),1):step['number']=number
            row['id']+=10000*attempt

class Contracts(unittest.TestCase):
    def test_changed_ids_resolve_original_suite_upload_and_bytes(self):
        api,run,source,jobs,*_=suite.fixture();enrich(run,jobs)
        actual,artifact,binding=suite.module.execution.suite_receipt(api,run,source,'ts','linux-x64-gnu')
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
        self.assertTrue(suite.module.execution.same_carried_execution(current,prior,run,3))
        current.pop('runner_id')
        self.assertFalse(suite.module.execution.same_carried_execution(current,prior,run,3))

    def test_changed_ids_in_unit_and_e2e_planners_and_cells(self):
        for file,scope in [('ci-unit-shadow-inputs.test.py','unit'),('ci-e2e-shadow-inputs.test.py','e2e')]:
            fixture=load('carried_'+scope,file)
            api,run,identity,jobs,*_=fixture.fixture();enrich(run,jobs)
            # E2E fixture exposes its scope-bound module wrapper.
            fixture.module.resolve(api,run,identity)

    def test_changed_id_missing_or_forged_execution_proof_rejects(self):
        for defect in ['source','attempt','runner','steps','step-time','step-status','step-number','name','failure']:
            with self.subTest(defect=defect):
                api,run,source,jobs,*_=suite.fixture();enrich(run,jobs)
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
                with self.assertRaises(ValueError):suite.module.execution.suite_receipt(api,run,source,'ts','linux-x64-gnu')

if __name__=='__main__':unittest.main(verbosity=2)
