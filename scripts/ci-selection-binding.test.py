"""Fixed selection and suite outputs must work without any clock fields."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
spec=importlib.util.spec_from_file_location('suite_output_fixture',Path(__file__).with_name('ci-suite-upload-outputs.test.py'))
f=importlib.util.module_from_spec(spec);spec.loader.exec_module(f)
m=f.module

def fixture():
    data=f.fixture.fixture();api,run,source,jobs,artifacts,bodies,_=data
    member=json.dumps({'sourceSHA':source['sha'],'sourceTree':source['tree'],'runId':run['id'],'attempt':1,'repositoryId':run['repository']['id']}).encode()
    stream=io.BytesIO()
    with zipfile.ZipFile(stream,'w') as z:z.writestr('selection.json',member)
    body=stream.getvalue();meta={'id':999,'name':'ci-suite-selection-2-1','expired':False,'workflow_run':{'id':run['id']},'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
    trusted=f.outputs(artifacts);artifacts[meta['name']]=meta;bodies[999]=body
    for attempt,rows in jobs.items():
        rows[:]=[r for r in rows if r['name']!='Type Check']
        rows.append({'id':998+attempt,'name':'Type Check','status':'completed','conclusion':'success','steps':[{'name':'Upload bound current suite selection','status':'completed','conclusion':'success'}]})
        for row in rows:
            row.update(run_id=run['id'],run_attempt=attempt,head_sha=run['head_sha'])
            row.pop('started_at',None);row.pop('completed_at',None)
            for step in row['steps']:step.pop('started_at',None);step.pop('completed_at',None)
    for row in artifacts.values():row.pop('created_at',None)
    binding={'artifact_id':'999','artifact_digest':meta['digest'][7:],'run_attempt':'1'}
    return data,trusted,binding,member

class Contracts(unittest.TestCase):
    def test_selection_and_six_suites_use_fixed_outputs_without_clocks(self):
        data,suites,selection,member=fixture()
        with tempfile.TemporaryDirectory() as tmp:
            target=Path(tmp)/'selection.json'
            result=m.collect(*data[:3],Path(tmp)/'receipts',hashlib.sha256(member).hexdigest(),target,suites,selection)
            self.assertEqual(len(result),6);self.assertEqual(target.read_bytes(),member)
    def test_selection_wrong_binding_or_latest_failure_rejects(self):
        for defect in ['id','digest','attempt','unknown','missing','failed']:
            data,_,bound,member=fixture()
            if defect=='id':bound['artifact_id']='888'
            elif defect=='digest':bound['artifact_digest']='e'*64
            elif defect=='attempt':bound['run_attempt']='2'
            elif defect=='unknown':bound['job']='self-asserted'
            elif defect=='missing':bound.pop('artifact_id')
            else:next(r for r in data[3][3] if r['name']=='Type Check')['conclusion']='failure'
            with self.subTest(defect=defect),self.assertRaises(ValueError):
                m.original_selection(*data[:3],hashlib.sha256(member).hexdigest(),bound)

if __name__=='__main__':unittest.main()
