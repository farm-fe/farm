"""Baseline lookup uses real ZIP verification and independent suite discovery."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

baseline=load('baseline_lookup_contract','ci-suite-baseline.py')
fixtures=load('baseline_zip_fixture','ci-execution-evidence.test.py')

class Contracts(unittest.TestCase):
    def setUp(self):
        self.args,self.discovery=fixtures.fixture()
        self.original,self.run,self.repository,self.ledger,self.artifact,self.jobs,self.source,*_=self.args
        self.current=copy.deepcopy(self.source);self.current['sha']='e'*40;self.current['tree']='f'*40
        self.env={'GITHUB_REPOSITORY_ID':'1','GITHUB_RUN_ID':'9','GITHUB_RUN_ATTEMPT':'2'}
        self.calls=[];self.pages=False;self.candidates=[{'id':2,'run_attempt':1}]
        self.signed_expected={'runId':2,'attempt':1,'callerWorkflow':self.run['path'],'sourceSHA':self.source['sha'],'repositoryId':1}
        for job in self.jobs:
            job.update(run_attempt=1,head_sha=self.source['sha'])
            for abi,platform in baseline.acceptance.PLATFORMS.items():
                if job['name'] in ['Typescript Test ('+platform+')','Examples Test ('+platform+')']:
                    job['name']={'linux-x64-gnu':'Primary Linux','darwin-arm64':'Primary macOS','win32-x64-msvc':'Primary Windows'}[abi]+' / '+job['name']
        self.signed_rows={'planner':{}}
        for cell in self.ledger['coverage']['cells']:
            cell.update(chain='ci',acceptanceInherited=False)
        self.original.pack()
        for cell in self.ledger['coverage']['cells']:
            label='ts' if cell['kind']=='ts' else 'browser'
            role=('Typescript' if label=='ts' else 'Examples')+' Test ('+baseline.acceptance.PLATFORMS[cell['abi']]+')'
            job=next(job for job in self.jobs if job['name'].endswith(' / '+role))
            meta=self.original.rows[cell['artifactName']]
            cell['producerBinding']={'runAttempt':1,'jobId':job['id'],'artifactId':meta['id'],'artifactDigest':meta['digest'],'profile':'ci','workflowPath':self.run['path']}
            self.signed_rows[label+':'+cell['abi']]=self.row(meta,job)
        self.original.pack()
        producer=next(job for job in self.jobs if job['name']==baseline.acceptance.PRODUCER)
        self.signed_rows['execution-baseline']=self.row(self.artifact,producer)
    def row(self,meta,job):
        return {'artifactId':meta['id'],'artifactDigest':meta['digest'],'artifactName':meta['name'],
                'originAttempt':1,'jobId':job['id'],'producer':job['name'],'uploadStep':job['steps'][0]['name']}
    def proof(self,transport,run,source):
        # Explicit offline signature seam; production uses pinned CLI verification.
        latest=max(row['run_attempt'] for row in self.candidates if row['id']==run['id'])
        if run['run_attempt']!=self.signed_expected['attempt'] or run['run_attempt']!=latest:raise ValueError('No signed current attempt')
        return self.signed_expected,self.signed_rows
    def get(self,route):
        self.calls.append(route)
        if route=='':return self.repository
        if route=='/branches/main':return {'commit':{'sha':self.source['sha']}}
        if route.startswith('/actions/workflows/ci.yaml/runs?'):return {'workflow_runs':self.candidates}
        if route=='/actions/runs/2/attempts/2':return {**self.run,'run_attempt':2}
        if route=='/actions/runs/2/attempts/1':return self.run
        if '/jobs?' in route:
            if not self.pages:return {'jobs':[{**job,'run_attempt':2 if '/attempts/2/' in route else 1} for job in self.jobs]}
            if route.endswith('&page=1'):return {'jobs':[{'id':1000+i,'name':'unrelated-'+str(i)} for i in range(100)]}
            return {'jobs':self.jobs}
        return self.original.get(route)
    def archive(self,identity):return self.original.archive(identity)
    def lookup(self,ancestor=True):
        return baseline.lookup(self,self.current,self.env,read_source=lambda _:self.source,is_ancestor=lambda *_:ancestor,
                               suite_discovery=lambda *_:copy.deepcopy(self.discovery),proof_provider=self.proof)
    def test_success_verifies_original_archive_and_six_receipts_before_current_run_binding(self):
        result=self.lookup()
        self.assertEqual(result['sourceSHA'],self.current['sha']);self.assertEqual(result['attempt'],2)
        self.assertEqual(result['baselineEvidence']['artifactDigest'],self.artifact['digest'])
        self.assertFalse(any('/artifacts?' in route for route in self.calls))
    def test_jobs_pagination_collects_required_producer_and_suite_uploads(self):
        self.pages=True;self.assertEqual(self.lookup()['baselineSHA'],self.source['sha'])
        self.assertTrue(any(route.endswith('&page=2') for route in self.calls))
    def test_missing_artifact_or_receipt_is_not_a_successful_baseline(self):
        self.original.rows.pop(next(iter(self.original.rows)))
        with self.assertRaises(ValueError):self.lookup()
        self.original.pack();self.artifact['expired']=True
        with self.assertRaises(ValueError):self.lookup()
    def test_wrong_attempt_source_policy_or_non_ancestor_never_authorizes_selection(self):
        self.run['run_attempt']=2
        with self.assertRaises(ValueError):self.lookup()
        self.run['run_attempt']=1;self.current['policyHash']='0'*64
        with self.assertRaisesRegex(ValueError,'policy'):self.lookup()
        self.current['policyHash']=self.source['policyHash']
        with self.assertRaises(ValueError):self.lookup(ancestor=False)
    def test_consistent_short_receipts_and_ledger_cannot_omit_discovered_cases(self):
        cell=self.ledger['coverage']['cells'][0]
        for key in ['selectedIds','executedIds','caseCounts']:cell[key]=cell[key][:1]
        cell['executedCases']=1;self.original.pack()
        for cell in self.ledger['coverage']['cells']:
            label='ts' if cell['kind']=='ts' else 'browser'
            digest=self.original.rows[cell['artifactName']]['digest']
            self.signed_rows[label+':'+cell['abi']]['artifactDigest']=digest
            cell['producerBinding']['artifactDigest']=digest
        self.original.pack()
        self.signed_rows['execution-baseline']['artifactDigest']=self.artifact['digest']
        # All byte/binding commitments agree; independent discovery must still
        # reject omission, rather than an earlier stale-digest check doing so.
        with self.assertRaisesRegex(ValueError,'Runner suite selection differs from independent source discovery'):
            baseline.acceptance.verify_execution_ledger(self,self.run,self.repository,self.ledger,self.artifact,
                self.jobs,self.source,self.source,self.ledger['requiredJobs'],
                suite_discovery=lambda *_:copy.deepcopy(self.discovery),signed_proof=(self.signed_expected,self.signed_rows))
        with self.assertRaises(ValueError):self.lookup()
    def test_repository_permission_failure_propagates_to_full_execution_fallback(self):
        original=self.get
        def denied(route):
            if '/artifacts/' in route:raise PermissionError('fixture permission denied')
            return original(route)
        self.get=denied
        with self.assertRaises(ValueError):self.lookup()
    def test_missing_newer_proof_cannot_fall_back_to_same_run_old_signed_attempt(self):
        self.candidates=[{'id':2,'run_attempt':2},{'id':2,'run_attempt':1}]
        with self.assertRaises(ValueError):self.lookup()
        self.assertTrue(any('/attempts/2' in route for route in self.calls))
    def test_gate_only_rerun_uses_original_summary_ledger_not_a_renamed_latest_artifact(self):
        self.candidates=[{'id':2,'run_attempt':2}]
        self.signed_expected['attempt']=2
        result=self.lookup()
        self.assertEqual(result['baselineEvidence']['runAttempt'],1)
        self.assertEqual(result['baselineEvidence']['observedAttempt'],2)
        self.assertFalse(any('name=farm-ci-execution-baseline-2-2' in route for route in self.calls))
        self.assertIn('/actions/artifacts/3',self.calls)

    def test_clock_fields_absent_or_arbitrary_do_not_change_signed_result(self):
        for job in self.jobs:
            job.pop('started_at',None);job['completed_at']='not-a-clock'
            for step in job['steps']:
                step.pop('started_at',None);step['completed_at']='different time zone'
        self.artifact.pop('created_at',None)
        for row in self.original.rows.values():row['created_at']='2099-01-01T00:00:00+12:00'
        self.assertEqual(self.lookup()['baselineSHA'],self.source['sha'])
    def test_unsigned_or_wrong_signed_identity_cannot_use_legacy_window(self):
        for defect in ['digest','job','attempt','role','missing']:
            original=copy.deepcopy(self.signed_rows)
            row=self.signed_rows['execution-baseline']
            if defect=='digest':row['artifactDigest']='sha256:'+'0'*64
            if defect=='job':row['jobId']+=999
            if defect=='attempt':row['originAttempt']=2
            if defect=='role':row['producer']='untrusted role'
            if defect=='missing':self.signed_rows.pop('execution-baseline')
            with self.subTest(defect=defect),self.assertRaises(ValueError):self.lookup()
            self.signed_rows=original

    def test_default_branch_change_is_rejected_before_any_archive_read(self):
        self.repository['default_branch']='other'
        with self.assertRaisesRegex(ValueError,'repository'):self.lookup()
        self.assertEqual(self.calls,[''])

if __name__=='__main__':unittest.main(verbosity=2)
