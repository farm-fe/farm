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
    def get(self,route):
        self.calls.append(route)
        if route=='':return self.repository
        if route=='/branches/main':return {'commit':{'sha':self.source['sha']}}
        if route.startswith('/actions/workflows/ci.yaml/runs?'):return {'workflow_runs':self.candidates}
        if route=='/actions/runs/2/attempts/2':return {**self.run,'run_attempt':2}
        if route=='/actions/runs/2/attempts/1':return self.run
        if '/jobs?' in route:
            if not self.pages:return {'jobs':self.jobs}
            if route.endswith('&page=1'):return {'jobs':[{'id':1000+i,'name':'unrelated-'+str(i)} for i in range(100)]}
            return {'jobs':self.jobs}
        return self.original.get(route)
    def archive(self,identity):return self.original.archive(identity)
    def lookup(self,ancestor=True):
        return baseline.lookup(self,self.current,self.env,read_source=lambda _:self.source,is_ancestor=lambda *_:ancestor,
                               suite_discovery=lambda *_:copy.deepcopy(self.discovery))
    def test_success_verifies_original_archive_and_six_receipts_before_current_run_binding(self):
        result=self.lookup()
        self.assertEqual(result['sourceSHA'],self.current['sha']);self.assertEqual(result['attempt'],2)
        self.assertEqual(result['baselineEvidence']['artifactDigest'],self.artifact['digest'])
        self.assertEqual(len([route for route in self.calls if '/artifacts?' in route]),7)
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
        with self.assertRaises(ValueError):self.lookup()
    def test_repository_permission_failure_propagates_to_full_execution_fallback(self):
        original=self.get
        def denied(route):
            if '/artifacts?' in route:raise PermissionError('fixture permission denied')
            return original(route)
        self.get=denied
        with self.assertRaises(ValueError):self.lookup()
    def test_missing_newer_ledger_continues_to_older_complete_candidate(self):
        self.candidates=[{'id':2,'run_attempt':2},{'id':2,'run_attempt':1}]
        result=self.lookup()
        self.assertEqual(result['baselineEvidence']['runAttempt'],1)
        self.assertTrue(any('/attempts/2' in route for route in self.calls))
    def test_default_branch_change_is_rejected_before_any_archive_read(self):
        self.repository['default_branch']='other'
        with self.assertRaisesRegex(ValueError,'repository'):self.lookup()
        self.assertEqual(self.calls,[''])

if __name__=='__main__':unittest.main(verbosity=2)
