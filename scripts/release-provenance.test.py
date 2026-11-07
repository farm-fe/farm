"""Offline release proofs: real files/API shapes, simulated signature boundary."""
# cspell:words importlib sigstore
import base64
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import zipfile
spec=importlib.util.spec_from_file_location('release_proof',Path(__file__).with_name('release-provenance.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


def fixture(scope='pending'):
    sha='a'*40;tree='b'*40;revision='c'*40
    environment={'GITHUB_ACTIONS':'true','RUNNER_ENVIRONMENT':'github-hosted','GITHUB_JOB':'sign',
        'GITHUB_EVENT_NAME':'push','GITHUB_REF':'refs/heads/main','GITHUB_REPOSITORY':m.REPOSITORY,
        'GITHUB_REPOSITORY_ID':str(m.REPOSITORY_ID),'GITHUB_WORKFLOW_REF':m.REPOSITORY+'/'+m.CALLERS[scope]+'@refs/heads/main',
        'SIGNER_WORKFLOW_SHA':revision,'SIGNER_WORKFLOW_REF':m.REPOSITORY+'/'+m.SIGNER+'@'+revision,
        'GITHUB_RUN_ID':'42','GITHUB_RUN_ATTEMPT':'1','GITHUB_SHA':sha,'SIGNER_CARGO_RESTORED':'false'}
    run={'id':42,'run_attempt':1,'head_sha':sha,'head_branch':'main','event':'push','path':m.CALLERS[scope],
         'repository':{'id':m.REPOSITORY_ID,'full_name':m.REPOSITORY},'head_repository':{'id':m.REPOSITORY_ID},
         'status':'in_progress','conclusion':None}
    functional=m.load('pending_fixture','incremental-release-functional.py')
    pending=functional.pending_record({'sha':sha,'tree':tree},{'runId':42,'attempt':1})
    buffer=io.BytesIO()
    with zipfile.ZipFile(buffer,'w') as archive:archive.writestr('release-functional-pending.json',json.dumps(pending))
    body=buffer.getvalue();jobs=[];artifacts={};outputs={}
    for index,(cell,role) in enumerate(m.cells(scope).items()):
        identity=100+index
        job=next((job for job in jobs if job['name']==role['producer']),None)
        if job is None:
            job={'id':200+index,'run_id':42,'run_attempt':1,'head_sha':sha,'name':role['producer'],
                 'status':'completed','conclusion':role.get('producerConclusion','success'),'steps':[]};jobs.append(job)
        job['steps'].append({'name':role['uploadStep'],'status':'completed','conclusion':'success'})
        artifact={'id':identity,'name':role['artifactName'].format(sourceSHA=sha,runId=42,originAttempt=1),
                  'workflow_run':{'id':42},'expired':False,'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
        artifacts[identity]=artifact
        outputs[cell]={'artifact_id':str(identity),'artifact_digest':artifact['digest'][7:],'run_attempt':'1'}
    template=b'uses: pinned@__REVIEWED_SIGNER_REVISION__'
    class API:
        def get(self,path):
            if path.startswith('/git/commits/'):return {'sha':sha,'tree':{'sha':tree}}
            if path.startswith('/contents/'):
                data=template.replace(b'__REVIEWED_SIGNER_REVISION__',revision.encode())
                return {'type':'file','path':m.CALLERS[scope],'encoding':'base64','size':len(data),'content':base64.b64encode(data).decode()}
            if '/jobs?' in path:return {'jobs':jobs}
            if path.startswith('/actions/artifacts/'):return artifacts[int(path.rsplit('/',1)[1])]
            return run
        def archive(self,identity):return body
    return API(),environment,outputs,{m.CALLERS[scope]:template},run,jobs,artifacts,pending


class Contracts(unittest.TestCase):
    def test_pending_failed_job_can_only_prove_exact_timeout_intent(self):
        api,env,outputs,templates,run,jobs,artifacts,_=fixture()
        body=m.prepare(api,env,'pending',outputs,templates)
        row=json.loads(body)['bindings'][0]
        self.assertEqual(row['artifactId'],100);self.assertEqual(jobs[0]['conclusion'],'failure')
        self.assertEqual(set(m.cells('pending')),{'pending'})
        self.assertNotIn('publicationAuthorized',json.loads(body))
        for defect in ['success','cancelled','skipped','bad-upload','digest','attempt']:
            api,env,outputs,templates,run,jobs,artifacts,_=fixture()
            if defect in ['success','cancelled','skipped']:jobs[0]['conclusion']=defect
            if defect=='bad-upload':jobs[0]['steps'][0]['conclusion']='failure'
            if defect=='digest':outputs['pending']['artifact_digest']='0'*64
            if defect=='attempt':outputs['pending']['run_attempt']='2'
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.prepare(api,env,'pending',outputs,templates)

    def test_failure_proof_cannot_authorize_frozen_public_or_cargo(self):
        for scope in ['frozen','public','cargo']:
            api,env,outputs,templates,run,jobs,_,_=fixture(scope)
            self.assertEqual(len(json.loads(m.prepare(api,env,scope,outputs,templates))['bindings']),len(m.cells(scope)))
            jobs[0]['conclusion']='failure'
            with self.subTest(scope=scope),self.assertRaises(ValueError):m.prepare(api,env,scope,outputs,templates)

    def test_dispatch_pr_foreign_repository_and_unpinned_signer_are_rejected(self):
        for field,value in [('GITHUB_EVENT_NAME','workflow_dispatch'),('GITHUB_EVENT_NAME','pull_request'),
                            ('GITHUB_REF','refs/pull/1/merge'),('GITHUB_REPOSITORY','other/farm'),
                            ('SIGNER_WORKFLOW_SHA','main'),('GITHUB_JOB','producer')]:
            api,env,outputs,templates,*_=fixture();env[field]=value
            with self.subTest(field=field,value=value),self.assertRaises(ValueError):m.prepare(api,env,'pending',outputs,templates)

    def test_pending_contents_cannot_claim_publication_or_other_attempt(self):
        for key,value in [('publicationAuthorized',True),('releaseAttempt',2),('reason','ordinary-job-failure')]:
            api,env,outputs,templates,run,jobs,artifacts,pending=fixture();pending[key]=value
            buffer=io.BytesIO()
            with zipfile.ZipFile(buffer,'w') as archive:archive.writestr('release-functional-pending.json',json.dumps(pending))
            body=buffer.getvalue();api.archive=lambda _:body
            artifacts[100].update(size_in_bytes=len(body),digest='sha256:'+hashlib.sha256(body).hexdigest())
            outputs['pending']['artifact_digest']=artifacts[100]['digest'][7:]
            with self.subTest(key=key),self.assertRaisesRegex(ValueError,'timeout intent'):m.prepare(api,env,'pending',outputs,templates)

    def test_carried_upload_can_retry_failed_signer_but_restore_requires_original_proof(self):
        api,env,outputs,templates,original,jobs,artifacts,_=fixture('cargo')
        current={**original,'run_attempt':2}
        env['GITHUB_RUN_ATTEMPT']='2'
        current_jobs=copy.deepcopy(jobs)
        for job in current_jobs:job.update(id=job['id']+1000,run_attempt=2)
        original_get=api.get
        def get(route):
            if route.endswith('/attempts/1'):return original
            if route.endswith('/attempts/2'):return current
            if '/attempts/1/jobs?' in route:return {'jobs':jobs}
            if '/attempts/2/jobs?' in route:return {'jobs':current_jobs}
            if '/artifacts?' in route:return {'total_count':0,'artifacts':[]}
            return original_get(route)
        api.get=get
        # Actual retained upload output, not restore-script output: no old proof
        # exists when the signer itself failed on the first attempt.
        result=json.loads(m.prepare(api,env,'cargo',outputs,templates))
        self.assertEqual(result['bindings'][0]['originAttempt'],1)
        self.assertEqual(result['bindings'][0]['jobId'],200)
        env['SIGNER_CARGO_RESTORED']='true'
        with self.assertRaisesRegex(ValueError,'no trusted signed producer'):
            m.prepare(api,env,'cargo',outputs,templates)
        # A real restore is accepted only with a previously verified immutable
        # binding, while the retry is still running. Signature is an explicit
        # offline seam here; artifact metadata and original job checks are real.
        without_proof=api.get
        api.get=lambda route: {'total_count':1,'artifacts':[{'id':999}]} if '/artifacts?' in route else without_proof(route)
        original_expected=m.expected_policy(original,{'sha':original['head_sha'],'tree':'b'*40},'cargo','c'*40)
        signed={'cargo':copy.deepcopy(result['bindings'][0])}
        def prior(client,observed,source,scope):
            self.assertEqual(observed['run_attempt'],1)
            return original_expected,copy.deepcopy(signed)
        self.assertTrue(m.prepare(api,env,'cargo',outputs,templates,prior_proof_provider=prior))
        signed['cargo']['artifactDigest']='sha256:'+'0'*64
        with self.assertRaises(ValueError):m.prepare(api,env,'cargo',outputs,templates,prior_proof_provider=prior)
        env['SIGNER_CARGO_RESTORED']='unknown'
        with self.assertRaisesRegex(ValueError,'restore mode'):
            m.prepare(api,env,'cargo',outputs,templates)

    def test_reviewed_cargo_restore_origin_is_from_upload_outcome_not_script_output(self):
        root=Path(__file__).parent.parent
        name='.github/workflows/cargo-source-release.yaml'
        source=(root/'scripts/release-provenance-templates/cargo'/name).read_text()
        self.assertIn("frozen_restored: ${{ steps.archive-upload.outcome != 'success' }}",source)
        self.assertNotIn('frozen_restored: ${{ steps.frozen.outputs.restored }}',source)
        self.assertIn("cargo_restored: ${{ needs.source.outputs.frozen_restored == 'true' }}",source)

    def test_no_clock_fields_are_required_for_any_signing_scope(self):
        for scope in m.CALLERS:
            api,env,outputs,templates,run,jobs,artifacts,_=fixture(scope)
            for job in jobs:
                job.update(started_at='not a timestamp',completed_at=None)
                for step in job['steps']:step.update(started_at='2099',completed_at='1900')
            for row in artifacts.values():row['created_at']='unrelated time zone'
            self.assertTrue(m.prepare(api,env,scope,outputs,templates))


class EnvelopeContracts(unittest.TestCase):
    def fixture(self):
        _,_,_,_,run,_,_,_=fixture('frozen')
        source={'sha':run['head_sha'],'tree':'b'*40}
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as archive:
            archive.writestr('ci-bindings.json',b'unsigned offline subject')
            archive.writestr('ci-bindings.sigstore.json',b'explicit simulated signature')
        body=output.getvalue()
        artifact={'id':400,'name':'farm-release-bindings-frozen-42-1','expired':False,
                  'workflow_run':{'id':42},'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
        state={'latest':{**run,'run_attempt':2,'status':'completed','conclusion':'failure'},
               'body':body,'artifact':artifact,'count':1,'reads':0}
        class API:
            def get(self,path):
                if path=='/actions/runs/42':
                    state['reads']+=1
                    return copy.deepcopy(state['latest'])
                if '/artifacts?' in path:return {'total_count':state['count'],'artifacts':[state['artifact']]}
                return state['artifact']
            def archive(self,identity):
                if identity!=400:raise AssertionError('Unexpected artifact')
                return state['body']
        return API(),run,source,state

    def verify(self,subject,bundle,expected):
        self.assertEqual(subject.read_bytes(),b'unsigned offline subject')
        self.assertEqual(bundle.read_bytes(),b'explicit simulated signature')
        self.assertEqual(expected['attempt'],1)
        return {cell:{'offlineOnly':True} for cell in expected['cells']}

    def test_original_frozen_proof_survives_later_failed_stage_without_changing_attempt(self):
        api,run,source,state=self.fixture()
        expected,rows=m.verified_run(api,run,source,'frozen',verify=self.verify,signer_sha='c'*40)
        self.assertEqual(expected['attempt'],1)
        self.assertEqual(set(rows),{'seal','candidates'})
        self.assertEqual(state['reads'],2)

    def test_original_frozen_proof_is_readable_while_retry_is_running_but_not_public_success(self):
        api,run,source,state=self.fixture()
        state['latest'].update(status='in_progress',conclusion=None)
        expected,_=m.verified_run(api,run,source,'frozen',verify=self.verify,signer_sha='c'*40)
        self.assertEqual(expected['attempt'],1)
        with self.assertRaisesRegex(ValueError,'current authoritative attempt'):
            m.verified_run(api,run,source,'public',verify=self.verify,signer_sha='c'*40)

    def test_envelope_tampering_ambiguity_source_change_and_concurrent_rerun_reject(self):
        for defect in ['missing','duplicate','bytes','expired','name','run','source','during-verification','inventory']:
            api,run,source,state=self.fixture()
            if defect in ['missing','duplicate']:state['count']=0 if defect=='missing' else 2
            if defect=='bytes':state['body']+=b'changed'
            if defect=='expired':state['artifact']['expired']=True
            if defect=='name':state['artifact']['name']='foreign'
            if defect=='run':state['artifact']['workflow_run']['id']=43
            if defect=='source':state['latest']['head_sha']='d'*40
            def verify(*args):
                rows=self.verify(*args)
                if defect=='during-verification':state['latest']['run_attempt']=3
                if defect=='inventory':return {'pending':{}}
                return rows
            with self.subTest(defect=defect),self.assertRaises(ValueError):
                m.verified_run(api,run,source,'frozen',verify=verify,signer_sha='c'*40)

    def test_signature_rejection_has_no_unsigned_fallback(self):
        api,run,source,_=self.fixture()
        def reject(*args):raise ValueError('cryptographic verification failed')
        with self.assertRaisesRegex(ValueError,'cryptographic verification failed'):
            m.verified_run(api,run,source,'frozen',verify=reject,signer_sha='c'*40)


if __name__=='__main__':unittest.main(verbosity=2)
