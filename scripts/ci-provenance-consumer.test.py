"""Real ZIP bytes with explicitly simulated signature verification boundary."""
# cspell:words importlib
import hashlib
import importlib.util
import io
from pathlib import Path
import unittest
import zipfile
spec=importlib.util.spec_from_file_location('consumer',Path(__file__).with_name('ci-provenance-consumer.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


class Contracts(unittest.TestCase):
    def fixture(self):
        source={'sha':'a'*40,'tree':'b'*40}
        run={'id':42,'run_attempt':3,'path':m.signer.CALLERS['unit'],'head_sha':source['sha'],'head_branch':'main',
             'event':'push','status':'completed','conclusion':'success','repository':{'id':m.signer.REPOSITORY_ID,'full_name':m.signer.REPOSITORY},'head_repository':{'id':m.signer.REPOSITORY_ID}}
        buffer=io.BytesIO()
        with zipfile.ZipFile(buffer,'w') as archive:
            archive.writestr('ci-bindings.json',b'not an authenticated fixture manifest')
            archive.writestr('ci-bindings.sigstore.json',b'explicitly simulated bundle')
        body=buffer.getvalue();artifact={'id':123,'name':'farm-ci-bindings-unit-42-3','expired':False,'workflow_run':{'id':42},'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
        state={'latest':run,'count':1,'body':body,'artifact':artifact,'reads':0}
        class API:
            def get(self,path):
                if path=='/actions/runs/42':
                    state['reads']+=1
                    return state['latest']
                if '/artifacts?' in path:return {'total_count':state['count'],'artifacts':[state['artifact']]}
                return state['artifact']
            def archive(self,identity):return state['body']
        return source,run,state,API()

    def verify_fixture(self,manifest,bundle,expected):
        self.assertEqual(manifest.read_bytes(),b'not an authenticated fixture manifest')
        self.assertEqual(bundle.read_bytes(),b'explicitly simulated bundle')
        self.assertEqual(expected['attempt'],3)
        return {'planner':{'fixture':True}}

    def test_real_archive_reaches_explicit_verifier_with_exact_current_attempt(self):
        source,run,state,api=self.fixture()
        expected,rows=m.load_verified_run(api,run,source,'c'*40,verify=self.verify_fixture)
        self.assertEqual(expected['sourceSHA'],source['sha']);self.assertEqual(rows,{'planner':{'fixture':True}})
        self.assertEqual(state['reads'],2)

    def test_missing_ambiguous_tampered_or_new_attempt_proof_rejects(self):
        for defect in ['missing','duplicate','bytes','expired','name','later-success','later-failure','during-verification']:
            source,run,state,api=self.fixture()
            if defect in ['missing','duplicate']:state['count']=0 if defect=='missing' else 2
            if defect=='bytes':state['body']+=b'tampered'
            if defect=='expired':state['artifact']['expired']=True
            if defect=='name':state['artifact']['name']='foreign-proof'
            if defect.startswith('later-'):state['latest']={**run,'run_attempt':4,'conclusion':'failure' if defect=='later-failure' else 'success'}
            def verify(*args):
                result=self.verify_fixture(*args)
                if defect=='during-verification':state['latest']={**run,'run_attempt':4}
                return result
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.load_verified_run(api,run,source,'c'*40,verify=verify)

    def test_verification_failure_cannot_become_unsigned_acceptance(self):
        source,run,state,api=self.fixture()
        def reject(*args):raise ValueError('cryptographic signature rejected')
        with self.assertRaisesRegex(ValueError,'signature rejected'):m.load_verified_run(api,run,source,'c'*40,verify=reject)
        with self.assertRaisesRegex(ValueError,'inventory'):m.load_verified_run(api,run,source,'c'*40,verify=lambda *args:{})


if __name__=='__main__':unittest.main()
