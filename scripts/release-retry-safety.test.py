"""Read-only attempt replay: only proven pre-freeze failures can be rebuilt."""
import copy
import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('retry_safety',Path(__file__).with_name('release-retry-safety.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)

class RetrySafety(unittest.TestCase):
    def fixture(self, attempts=2):
        options=dict(run_id=123,source_sha='a'*40,current_attempt=attempts,workflow='.github/workflows/release.yaml',producer='producer',freeze_step='freeze',publisher='publisher',events=['push'])
        data={'':{'id':5}}
        for attempt in range(1,attempts):
            route='/actions/runs/123/attempts/'+str(attempt)
            data[route]={'id':123,'run_attempt':attempt,'path':options['workflow'],'head_sha':'a'*40,'status':'completed','event':'push','repository':{'id':5},'head_repository':{'id':5}}
            data[route+'/jobs?per_page=100']={'total_count':2,'jobs':[
                {'run_id':123,'name':'producer','status':'completed','conclusion':'failure','steps':[{'name':'freeze','status':'completed','conclusion':'skipped'}]},
                {'run_id':123,'name':'publisher','status':'completed','conclusion':'skipped','steps':[]}]}
        return options,data
    def test_all_prior_attempts_prove_neither_freeze_nor_publish_executed(self):
        options,data=self.fixture(4)
        result=module.verify_unfrozen_retry(data.__getitem__,**options)
        self.assertEqual([row['attempt'] for row in result],[1,2,3])
    def test_started_failed_cancelled_or_missing_publisher_is_not_safe(self):
        options,data=self.fixture();route='/actions/runs/123/attempts/1/jobs?per_page=100'
        for conclusion in ['success','failure','cancelled','timed_out',None]:
            changed=copy.deepcopy(data);changed[route]['jobs'][1]['conclusion']=conclusion
            with self.assertRaises(ValueError):module.verify_unfrozen_retry(changed.__getitem__,**options)
        changed=copy.deepcopy(data);changed[route]['jobs']=changed[route]['jobs'][:1];changed[route]['total_count']=1
        with self.assertRaisesRegex(ValueError,'missing or ambiguous'):module.verify_unfrozen_retry(changed.__getitem__,**options)
    def test_any_frozen_or_uncertain_candidate_requires_original_bytes(self):
        options,data=self.fixture();route='/actions/runs/123/attempts/1/jobs?per_page=100'
        for conclusion in ['success','failure','cancelled',None]:
            changed=copy.deepcopy(data);changed[route]['jobs'][0]['steps'][0]['conclusion']=conclusion
            with self.assertRaisesRegex(ValueError,'freeze may have executed'):module.verify_unfrozen_retry(changed.__getitem__,**options)
        changed=copy.deepcopy(data);changed[route]['jobs'][0]['conclusion']='success'
        with self.assertRaisesRegex(ValueError,'successful producer'):module.verify_unfrozen_retry(changed.__getitem__,**options)
    def test_prior_attempt_source_repo_or_workflow_mismatch_rejected(self):
        options,data=self.fixture();route='/actions/runs/123/attempts/1'
        for key,value in [('head_sha','b'*40),('path','other.yaml'),('event','pull_request'),('run_attempt',2),('status','in_progress'),('head_repository',{'id':9})]:
            changed=copy.deepcopy(data);changed[route][key]=value
            with self.assertRaises(ValueError):module.verify_unfrozen_retry(changed.__getitem__,**options)
    def test_truncated_listing_missing_steps_and_attempt_bound_rejected(self):
        options,data=self.fixture();route='/actions/runs/123/attempts/1/jobs?per_page=100'
        changed=copy.deepcopy(data);changed[route]['total_count']=101
        with self.assertRaisesRegex(ValueError,'incomplete'):module.verify_unfrozen_retry(changed.__getitem__,**options)
        changed=copy.deepcopy(data);del changed[route]['jobs'][1]['steps']
        with self.assertRaisesRegex(ValueError,'publisher may have executed'):module.verify_unfrozen_retry(changed.__getitem__,**options)
        with self.assertRaisesRegex(ValueError,'bound'):module.verify_unfrozen_retry(data.__getitem__,**{**options,'current_attempt':21})
    def test_returned_job_attempt_and_source_are_checked_when_present(self):
        options,data=self.fixture();route='/actions/runs/123/attempts/1/jobs?per_page=100'
        for key,value in [('run_attempt',2),('head_sha','b'*40)]:
            changed=copy.deepcopy(data);changed[route]['jobs'][1][key]=value
            with self.assertRaisesRegex(ValueError,'source or attempt differs'):module.verify_unfrozen_retry(changed.__getitem__,**options)
    def test_earlier_uncertain_attempt_cannot_be_hidden_by_latest_safe_failure(self):
        options,data=self.fixture(3);data['/actions/runs/123/attempts/1/jobs?per_page=100']['jobs'][1]['conclusion']='failure'
        with self.assertRaises(ValueError):module.verify_unfrozen_retry(data.__getitem__,**options)

if __name__=='__main__':unittest.main(verbosity=2)
