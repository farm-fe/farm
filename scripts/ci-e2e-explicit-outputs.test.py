"""All current E2E cells bind to trusted fixed caller outputs, without clocks."""
# cspell:words importlib
import copy
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('e2e_fixture',Path(__file__).with_name('ci-e2e-shadow-inputs.test.py'))
f=importlib.util.module_from_spec(spec);spec.loader.exec_module(f)
m=f.module

def case():
    data=f.fixture(); values={}
    for attempt,jobs in data[3].items():
        for job in jobs:
            job.update(head_sha=data[1]['head_sha'],run_attempt=attempt)
            job.pop('started_at');job.pop('completed_at')
            for step in job['steps']:step.pop('started_at');step.pop('completed_at')
            if attempt==2:job['id']+=10000
    for index,(cell,*_) in enumerate([('planner',),*m.CELLS]):
        meta=list(data[4].values())[index]
        values[cell]={'artifact_id':str(meta['id']),'artifact_digest':meta['digest'][7:],'run_attempt':meta['name'].rsplit('-',1)[1]}
        meta.pop('created_at')
    return data,values

class Contracts(unittest.TestCase):
    def resolve(self,data,values):
        planner,cells=m.explicit_upload_outputs({'CI_E2E_UPLOADS':json.dumps(values)})
        return m.resolve(*data[:3],upload_outputs=cells,planner_output=planner,require_explicit=True)
    def test_all_24_cells_and_planner_without_any_clock_fields(self):
        data,values=case();plan,rows,binding=self.resolve(data,values)
        self.assertEqual(len(rows),24)
        self.assertEqual(binding['plannerAttempt'],1)
        self.assertEqual({r['runAttempt'] for r in rows},{'1','2'})
    def test_each_cell_missing_or_swapped_rejects(self):
        for cell in ['planner',*[row[0] for row in m.CELLS]]:
            for defect in ['missing','digest','attempt']:
                data,values=case()
                if defect=='missing':values.pop(cell)
                elif defect=='digest':values[cell]['artifact_digest']='f'*64
                else:values[cell]['run_attempt']='3'
                with self.subTest(cell=cell,defect=defect),self.assertRaises(ValueError):self.resolve(data,values)
    def test_missing_unknown_binding_fields_reject(self):
        for defect in ['extra-cell','extra-field','missing-field','wrong-type']:
            data,values=case()
            if defect=='extra-cell':values['other']=values['planner']
            elif defect=='extra-field':values['planner']['job']='self-asserted'
            elif defect=='missing-field':values['planner'].pop('artifact_digest')
            else:values['planner']['artifact_id']=1
            with self.subTest(defect=defect),self.assertRaises(ValueError):self.resolve(data,values)
    def test_latest_failure_no_receipt_fallback(self):
        data,values=case();data[3][2][1]['conclusion']='failure'
        with self.assertRaises(ValueError):self.resolve(data,values)
    def test_production_binding_inventory_cannot_use_legacy_fallback(self):
        data,_=case()
        with self.assertRaises(ValueError):m.resolve(*data[:3],require_explicit=True)
        data,values=case()
        planner,cells=m.explicit_upload_outputs({'CI_E2E_UPLOADS':json.dumps(values)})
        cells[next(iter(cells))]=None
        with self.assertRaises(ValueError):
            m.resolve(*data[:3],upload_outputs=cells,planner_output=planner,require_explicit=True)

if __name__=='__main__':unittest.main()
