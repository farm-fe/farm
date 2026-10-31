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
                             "size_in_bytes": len(bodies[number]), "digest": 'sha256:'+hashlib.sha256(bodies[number]).hexdigest()} for number in runs}
        artifacts[110] = {'id':110,'name':'performance-pr-summary-1','expired':False,'workflow_run':{'id':10},
                          'created_at':'2','size_in_bytes':len(bodies[110]),'digest':'sha256:'+hashlib.sha256(bodies[110]).hexdigest()}
        for number, names in [(10, ['Incremental CI plan (shadow only)', *module.FUNCTIONAL]),
                              (20, ['Incremental Unit CI plan (shadow only)', 'Check', *['Test - '+name for name in module.PLATFORMS]])]:
            jobs[number] = [{"name": name, "status": "completed", "conclusion": "success", "started_at":"1", "completed_at":"3"} for name in names]
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


if __name__=='__main__':unittest.main(verbosity=2)
