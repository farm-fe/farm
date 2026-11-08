"""Production same-run functional upload reader with isolated API/ZIP fixtures."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import zipfile
spec=importlib.util.spec_from_file_location('functional_upload',Path(__file__).with_name('release-functional-upload.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


def fixture(origin=1):
    run={'id':50,'run_attempt':2,'head_sha':'a'*40,'head_branch':'main','event':'push','path':'.github/workflows/incremental-release.yaml','repository':{'id':507542208,'full_name':'farm-fe/farm'},'head_repository':{'id':507542208}}
    jobs={attempt:[{'id':100+attempt,'run_id':50,'run_attempt':attempt,'head_sha':run['head_sha'],'name':m.PRODUCER,'status':'completed','conclusion':'success','steps':[{'name':m.UPLOAD,'status':'completed','conclusion':'success'}]}] for attempt in [1,2]}
    out=io.BytesIO()
    with zipfile.ZipFile(out,'w') as z:z.writestr('functional-evidence.json',json.dumps({'ci':{'attempt':3}}))
    body=out.getvalue();artifact={'id':90,'name':f'release-functional-50-{origin}','workflow_run':{'id':50,'head_sha':run['head_sha']},'expired':False,'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
    binding={'artifact_id':'90','artifact_digest':artifact['digest'][7:],'run_attempt':str(origin)}
    class API:
        def get(self,route):
            if '/attempts/' in route:
                attempt=int(route.split('/attempts/')[1].split('/')[0])
                return {'jobs':jobs[attempt]} if '/jobs?' in route else {**run,'run_attempt':attempt}
            if route=='/actions/artifacts/90':return artifact
            raise ValueError('Unknown artifact')
        def archive(self,number):return body
    return API(),run,binding,artifact,jobs


class Contracts(unittest.TestCase):
    def test_original_upload_carries_or_new_upload_uses_new_attempt(self):
        for origin in [1,2]:
            api,run,binding,*_=fixture(origin)
            self.assertEqual(m.restore(api,run,binding),{'ci':{'attempt':3}})

    def test_missing_future_forged_or_failed_bindings_reject(self):
        for defect in ['empty','future','id','digest','source','name','failed','cancelled','skipped','expired','corrupt','step','ambiguous']:
            api,run,binding,artifact,jobs=fixture()
            if defect=='empty':binding['run_attempt']=''
            elif defect=='future':binding['run_attempt']='3'
            elif defect=='id':binding['artifact_id']='91'
            elif defect=='digest':binding['artifact_digest']='0'*64
            elif defect=='source':artifact['workflow_run']['head_sha']='b'*40
            elif defect=='name':artifact['name']='release-functional-50-2'
            elif defect=='failed':jobs[2][0]['conclusion']='failure'
            elif defect in ['cancelled','skipped']:jobs[2][0]['conclusion']=defect
            elif defect=='expired':artifact['expired']=True
            elif defect=='corrupt':api.archive=lambda number:b'corrupt ZIP'
            elif defect=='step':jobs[1][0]['steps'][0]['conclusion']='failure'
            else:jobs[2].append(copy.deepcopy(jobs[2][0]))
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.restore(api,run,binding)


if __name__=='__main__':unittest.main()
