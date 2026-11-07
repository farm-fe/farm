"""Fixed Unit upload outputs: offline real boundary and mixed-attempt contracts."""
# cspell:words importlib
import copy
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('unit_fixture', Path(__file__).with_name('ci-unit-shadow-inputs.test.py'))
f = importlib.util.module_from_spec(spec); spec.loader.exec_module(f)
m = f.module


def outputs(items):
    result = {}
    for key, suffix in [('planner', 'impact-plan-'), *[(cell, 'impact-cell-') for cell, *_ in m.CELLS]]:
        if key == 'planner': row = next(v for n,v in items.items() if n.startswith(suffix))
        else:
            _, _, job, platform = next(x for x in m.CELLS if x[0] == key)
            row = next(v for n,v in items.items() if '-' + job + '-' + platform + '-' in n)
        result[key] = {'artifact_id':str(row['id']), 'artifact_digest':row['digest'].removeprefix('sha256:'), 'run_attempt':row['name'].rsplit('-',1)[1]}
    return result


class Contracts(unittest.TestCase):
    def resolve(self, data, values):
        api,run,identity,*_ = data
        plan, cells = m.unit_upload_outputs({'CI_UNIT_UPLOADS':json.dumps(values)})
        return m.resolve(api,run,identity,upload_outputs=cells,planner_output=plan)

    def test_late_metadata_and_mixed_attempts_use_exact_action_outputs(self):
        data=f.fixture(); values=outputs(data[4])
        for row in data[4].values(): row['created_at']='2026-11-02T09:00:00Z'
        with self.assertRaisesRegex(ValueError,'outside its fixed upload'): m.resolve(*data[:3])
        plan,rows,binding=self.resolve(data,values)
        self.assertEqual(binding['plannerAttempt'],1)
        self.assertEqual(binding['cells']['rust-tests:ci-test:linux-x64-gnu']['producerAttempt'],2)
        self.assertEqual(len(rows),4)

    def test_actual_mac_upload_observation_rejects_time_but_accepts_fixed_output(self):
        observed=json.loads(Path(__file__).with_name('fixtures').joinpath('unit-upload-boundary-observation.json').read_text())
        with self.assertRaisesRegex(ValueError,'outside its fixed upload'):
            m.execution.upload_binding(observed['job'],'Preserve actual cell receipt (shadow only)',observed['artifact'])
        m.execution.upload_binding(observed['job'],'Preserve actual cell receipt (shadow only)',observed['artifact'],observed['uploadOutputs'])

    def test_reusable_platform_job_names_preserve_original_cell_identity(self):
        data=f.fixture(); values=outputs(data[4])
        for jobs in data[3].values():
            for job in jobs:
                if job['name'].startswith('Test - '): job['name']='cargo-test-caller / '+job['name']
        self.assertEqual(len(self.resolve(data,values)[1]),4)

    def test_missing_unknown_and_malformed_outputs_reject(self):
        for defect in ['missing-cell','extra-cell','missing-field','extra-field','id','digest','attempt','shape']:
            data=f.fixture(); values=outputs(data[4]); row=values['rust-tests:ci-test:darwin-arm64']
            if defect=='missing-cell': values.pop('planner')
            elif defect=='extra-cell': values['other']=row
            elif defect=='missing-field': row.pop('artifact_digest')
            elif defect=='extra-field': row['self_claim']='trusted'
            elif defect=='shape': values=[]
            else: row[{'id':'artifact_id','digest':'artifact_digest','attempt':'run_attempt'}[defect]]=0
            with self.subTest(defect=defect),self.assertRaises(ValueError): self.resolve(data,values)

    def test_valid_shape_wrong_binding_and_platform_swap_reject(self):
        for defect in ['id','digest','attempt','platform','planner-attempt']:
            data=f.fixture(); values=outputs(data[4]); key='rust-tests:ci-test:darwin-arm64'
            if defect=='platform': values[key]=copy.deepcopy(values['rust-tests:ci-test:win32-x64-msvc'])
            elif defect=='planner-attempt': values['planner']['run_attempt']='2'
            else: values[key][{'id':'artifact_id','digest':'artifact_digest','attempt':'run_attempt'}[defect]]={'id':'999','digest':'f'*64,'attempt':'2'}[defect]
            with self.subTest(defect=defect),self.assertRaises(ValueError): self.resolve(data,values)

    def test_failure_job_step_and_corrupt_bytes_still_reject(self):
        for defect in ['job','step','bytes']:
            data=f.fixture(); values=outputs(data[4]); job=next(j for j in data[3][2] if j['name']=='Test - macos-latest')
            if defect=='job': job['conclusion']='failure'
            elif defect=='step':
                for rows in data[3].values():
                    for original in rows:
                        if original['name']=='Test - macos-latest': original['steps'][0]['conclusion']='failure'
            else: data[5][int(values['rust-tests:ci-test:darwin-arm64']['artifact_id'])]+=b'bad'
            with self.subTest(defect=defect),self.assertRaises(ValueError): self.resolve(data,values)

if __name__=='__main__': unittest.main()
