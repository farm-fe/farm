"""Native audit uses explicit upload outputs; it grants no byte reuse authority."""
import copy
import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('native_audit',Path(__file__).with_name('ci-native-evidence.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

def case():
    run={'id':1,'run_attempt':2,'head_sha':'a'*40,'path':'.github/workflows/ci.yaml','event':'push','repository':{'id':3},'head_repository':{'id':3}}
    bindings={};jobs={1:[],2:[]};artifacts={}
    for index,abi in enumerate(m.impact.TARGETS):
        prefix='Build and Upload User Artifacts - ' if abi in m.impact.PRIMARY else 'Build and Upload Release Artifacts - '
        bindings[abi]={'artifact_id':str(100+index),'artifact_digest':'b'*64,'run_attempt':'1'}
        for attempt in jobs:
            jobs[attempt].append({'id':1000*attempt+index,'run_id':1,'run_attempt':attempt,'head_sha':run['head_sha'],'name':'Fixed / '+prefix+abi,'status':'completed','conclusion':'success','steps':[{'name':'Preserve bounded shadow producer context without duplicating native archives','status':'completed','conclusion':'success'}]})
        artifacts[100+index]={'id':100+index,'name':'native-shadow-'+run['head_sha']+'-'+abi+'-1','workflow_run':{'id':1},'expired':False,'digest':'sha256:'+'b'*64,'size_in_bytes':500}
    class API:
        def get(self,route):
            if '/attempts/' in route:
                attempt=int(route.split('/attempts/')[1].split('/')[0]);return {'jobs':jobs[attempt]} if '/jobs?' in route else {**run,'run_attempt':attempt}
            return artifacts[int(route.rsplit('/',1)[1])]
    return API(),run,bindings,jobs,artifacts

class Contracts(unittest.TestCase):
    def test_all_nine_no_clocks_changed_carried_ids(self):
        api,run,bindings,*_=case();rows=m.audit_native_bindings(api,run,run['head_sha'],bindings)
        self.assertEqual(len(rows),9);self.assertEqual({r['runAttempt'] for r in rows},{1})
        self.assertTrue(all(r['contextVerified'] is False for r in rows))
    def test_wrong_missing_or_ambiguous_proof_rejects(self):
        for defect in ['digest','id','abi','source','run','attempt','missing','extra','failure','duplicate','upload']:
            api,run,bindings,jobs,artifacts=case();abi=next(iter(bindings));meta=next(iter(artifacts.values()))
            if defect=='digest':meta['digest']='sha256:'+'c'*64
            elif defect=='id':meta['id']+=100
            elif defect in ['abi','source']:meta['name']='native-shadow-wrong'
            elif defect=='run':meta['workflow_run']['id']=9
            elif defect=='attempt':bindings[abi]['run_attempt']='2'
            elif defect=='missing':bindings.pop(abi)
            elif defect=='extra':bindings['unknown']=bindings[abi]
            elif defect=='failure':jobs[2][0]['conclusion']='failure'
            elif defect=='duplicate':jobs[1].append(copy.deepcopy(jobs[1][0]))
            else:jobs[1][0]['steps'][0]['conclusion']='failure'
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.audit_native_bindings(api,run,run['head_sha'],bindings)

if __name__=='__main__':unittest.main()
