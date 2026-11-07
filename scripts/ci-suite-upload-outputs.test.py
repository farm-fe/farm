"""Current Actions output bindings are independent of runner/server clock order."""
import copy
import importlib.util
import json
import re
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('upload_fixture', Path(__file__).with_name('ci-suite-attempts.test.py'))
fixture = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixture)
module = fixture.module


def outputs(artifacts):
    result = {abi: {} for abi in module.execution.PLATFORMS}
    for artifact in artifacts.values():
        abi = next(abi for abi in result if abi in artifact['name'])
        kind = 'ts' if '-suite-ts-' in artifact['name'] else 'examples-browser'
        result[abi][kind] = {'artifact_id':str(artifact['id']), 'artifact_digest':artifact['digest'][7:], 'run_attempt':'1'}
    return result


class Contracts(unittest.TestCase):
    def test_actual_observation_has_server_created_after_runner_next_step(self):
        data = json.loads(Path(__file__).with_name('fixtures').joinpath('suite-upload-clock-observation.json').read_text())
        self.assertEqual(data['artifact']['id'], 13139547429)
        self.assertEqual(data['artifact']['created_at'], '2026-11-02T12:03:18Z')
        self.assertTrue(any('12:03:17.6860270Z' in row and 'Artifact ID 13139547429' in row for row in data['runnerLogExcerpt']))
        self.assertTrue(any('12:03:17.7156970Z' in row and 'Run actions/download-artifact' in row for row in data['runnerLogExcerpt']))
        # These are observed log times, not invented API step timestamps.
        self.assertTrue(data['limitations'])

    def test_current_output_binding_accepts_clock_mismatch_and_preserves_original_attempt(self):
        transport,run,source,jobs,artifacts,*_ = fixture.fixture()
        trusted = outputs(artifacts)
        for artifact in artifacts.values(): artifact['created_at'] = '2026-10-31T01:03:01Z'
        with tempfile.TemporaryDirectory() as tmp:
            proofs = module.collect(transport,run,source,Path(tmp)/'direct',upload_outputs=trusted)
            self.assertEqual(len(proofs),6)
            self.assertEqual({row['runAttempt'] for row in proofs.values()},{1})
            with self.assertRaisesRegex(ValueError,'outside its fixed upload step'):
                module.collect(transport,run,source,Path(tmp)/'historical')

    def test_wrong_id_digest_attempt_and_missing_cell_fail_closed(self):
        for field, value in [('artifact_id','999999'),('artifact_digest','0'*64),('run_attempt','3')]:
            transport,run,source,jobs,artifacts,*_ = fixture.fixture()
            trusted=outputs(artifacts);trusted['linux-x64-gnu']['ts'][field]=value
            with tempfile.TemporaryDirectory() as tmp, self.assertRaises(ValueError):
                module.collect(transport,run,source,Path(tmp)/'out',upload_outputs=trusted)

    def test_direct_outputs_do_not_authorize_corrupt_bytes_or_absent_cells(self):
        for defect in ['bytes', 'cell']:
            transport,run,source,jobs,artifacts,bodies,_ = fixture.fixture();trusted=outputs(artifacts)
            if defect == 'bytes':
                artifact=next(row for row in artifacts.values() if '-suite-ts-' in row['name'] and 'linux-x64-gnu' in row['name'])
                bodies[artifact['id']] += b'corrupt'
            else: del trusted['linux-x64-gnu']['ts']
            with tempfile.TemporaryDirectory() as tmp, self.assertRaises((ValueError,KeyError)):
                module.collect(transport,run,source,Path(tmp)/'out',upload_outputs=trusted)

    def test_latest_failure_and_failed_fixed_upload_are_never_overridden(self):
        for target in ['producer','upload']:
            transport,run,source,jobs,artifacts,*_ = fixture.fixture();trusted=outputs(artifacts)
            for attempt in jobs:
                job=next(row for row in jobs[attempt] if row['name'].startswith('Primary Linux / Typescript'))
                if target=='producer': job['conclusion']='failure'
                else: job['steps'][0]['conclusion']='failure'
            with tempfile.TemporaryDirectory() as tmp, self.assertRaises(ValueError):
                module.collect(transport,run,source,Path(tmp)/'out',upload_outputs=trusted)

    def test_fixed_environment_map_requires_all_cells_and_valid_values(self):
        artifacts=fixture.fixture()[4];trusted=outputs(artifacts);env={}
        for abi,platform in [('linux-x64-gnu','LINUX'),('darwin-arm64','MACOS'),('win32-x64-msvc','WINDOWS')]:
            row={('ts' if kind=='ts' else 'browser')+'_'+field:value for kind,output in trusted[abi].items() for field,value in output.items()}
            env['FARM_SUITE_UPLOADS_'+platform]=json.dumps(row)
        self.assertEqual(module.current_upload_outputs(env),trusted)
        for value in ['{}','null','{"ts_artifact_id":"123"}']:
            bad={**env,'FARM_SUITE_UPLOADS_LINUX':value}
            with self.assertRaises(ValueError):module.current_upload_outputs(bad)
        bad=copy.deepcopy(env);row=json.loads(bad['FARM_SUITE_UPLOADS_LINUX']);row['ts_artifact_digest']='garbage'
        bad['FARM_SUITE_UPLOADS_LINUX']=json.dumps(row)
        with self.assertRaises(ValueError):module.current_upload_outputs(bad)

    def test_actual_complete_primary_outputs_replay_and_yaml_interface(self):
        data=json.loads(Path(__file__).with_name('fixtures').joinpath('primary-upload-outputs-observation.json').read_text())
        env={key:json.dumps(value) for key,value in data['outputs'].items()}
        resolved=module.current_upload_outputs(env)
        self.assertEqual(len(resolved),3)
        self.assertEqual(resolved['darwin-arm64']['ts']['artifact_id'],'13398437236')
        root=Path(__file__).parents[1]
        interface=(root/'.github/workflows/ci-primary-platform.yaml').read_text().split('    outputs:\n',1)[1].split('permissions:',1)[0]
        declared=set(re.findall(r'^      ([a-z_]+):$',interface,re.M))
        self.assertEqual(len(declared),19)
        self.assertIn('native_product_upload',declared)
        legacy = {key for key in declared if not key.startswith(('native_', 'docs_'))}
        for row in data['outputs'].values():self.assertEqual(set(row),legacy)
        spec=importlib.util.spec_from_file_location('shadow_outputs',Path(__file__).with_name('ci-e2e-shadow-inputs.py'))
        shadow=importlib.util.module_from_spec(spec);spec.loader.exec_module(shadow)
        observed=shadow.primary_upload_outputs({key.replace('FARM_SUITE_','CI_SHADOW_'):value for key,value in env.items()})
        self.assertEqual(observed['ts-tests:darwin-arm64']['artifact_id'],'13398601780')
        # Public schema extension never changes which exact suite bytes are selected.
        for abi,platform in [('linux-x64-gnu','LINUX'),('darwin-arm64','MACOS'),('win32-x64-msvc','WINDOWS')]:
            row=data['outputs']['FARM_SUITE_UPLOADS_'+platform]
            for label,kind in [('ts','ts'),('browser','examples-browser')]:
                self.assertEqual(resolved[abi][kind],{field:row[label+'_'+field] for field in ['artifact_id','artifact_digest','run_attempt']})

    def test_new_primary_fields_accept_missing_skipped_mac_docs_but_not_partial_or_forged(self):
        original=json.loads(Path(__file__).with_name('fixtures').joinpath('primary-upload-outputs-observation.json').read_text())['outputs']
        for defect in [None,'partial-docs','partial-native','unknown','native-digest','linux-missing-docs','mac-docs-present','proof-digest','proof-attempt']:
            rows=copy.deepcopy(original)
            for platform in ['LINUX','MACOS','WINDOWS']:
                row=rows['FARM_SUITE_UPLOADS_'+platform]
                for label in ['native','docs']:
                    if platform=='MACOS' and label=='docs':continue
                    row.update({label+'_shadow_artifact_id':'123',label+'_shadow_artifact_digest':'a'*64,label+'_shadow_run_attempt':'1'})
            for row in rows.values():row['native_product_upload']=json.dumps({'artifact_id':'123','artifact_digest':'b'*64,'run_attempt':'1'})
            rows['FARM_SUITE_UPLOADS_LINUX']['native_product_upload']=''
            mac=rows['FARM_SUITE_UPLOADS_MACOS']
            if defect=='proof-digest':mac['native_product_upload']=json.dumps({'artifact_id':'123','artifact_digest':'bad','run_attempt':'1'})
            elif defect=='proof-attempt':mac['native_product_upload']=json.dumps({'artifact_id':'123','artifact_digest':'b'*64,'run_attempt':'2'})
            elif defect=='partial-docs':mac['docs_shadow_artifact_id']=''
            elif defect=='partial-native':mac.pop('native_shadow_artifact_id')
            elif defect=='unknown':mac['unknown']=''
            elif defect=='native-digest':mac['native_shadow_artifact_digest']='bad'
            elif defect=='linux-missing-docs':
                for field in ['artifact_id','artifact_digest','run_attempt']:rows['FARM_SUITE_UPLOADS_LINUX'].pop('docs_shadow_'+field)
            elif defect=='mac-docs-present':
                mac.update({'docs_shadow_artifact_id':'123','docs_shadow_artifact_digest':'a'*64,'docs_shadow_run_attempt':'1'})
            env={key:json.dumps(value) for key,value in rows.items()}
            if defect:
                with self.subTest(defect=defect),self.assertRaises(ValueError):module.current_upload_outputs(env)
            else:self.assertEqual(len(module.current_upload_outputs(env)),3)

    def test_extended_outputs_reject_unknown_partial_or_malformed_fields(self):
        data=json.loads(Path(__file__).with_name('fixtures').joinpath('primary-upload-outputs-observation.json').read_text())
        for defect in ['unknown','partial','id','digest','attempt','null']:
            rows=copy.deepcopy(data['outputs']);row=rows['FARM_SUITE_UPLOADS_MACOS']
            if defect=='unknown':row['unreviewed_field']='1'
            elif defect=='partial':del row['ts_shadow_run_attempt']
            elif defect=='null':row['ts_shadow_artifact_id']=None
            else:row['ts_shadow_'+{'id':'artifact_id','digest':'artifact_digest','attempt':'run_attempt'}[defect]]='bad'
            with self.subTest(defect=defect),self.assertRaises(ValueError):module.current_upload_outputs({key:json.dumps(value) for key,value in rows.items()})
            spec=importlib.util.spec_from_file_location('shadow_negative',Path(__file__).with_name('ci-e2e-shadow-inputs.py'))
            shadow=importlib.util.module_from_spec(spec);spec.loader.exec_module(shadow)
            with self.subTest(shadow=defect),self.assertRaises(ValueError):shadow.primary_upload_outputs({key.replace('FARM_SUITE_','CI_SHADOW_'):json.dumps(value) for key,value in rows.items()})

    def test_complete_workflow_outputs_drive_actual_suite_collection(self):
        for defect in [None,'artifact_id','artifact_digest','run_attempt']:
            transport,run,source,jobs,artifacts,*_=fixture.fixture();trusted=outputs(artifacts);env={}
            for abi,platform in [('linux-x64-gnu','LINUX'),('darwin-arm64','MACOS'),('win32-x64-msvc','WINDOWS')]:
                row={('ts' if kind=='ts' else 'browser')+'_'+field:value for kind,output in trusted[abi].items() for field,value in output.items()}
                for label in ['ts','browser']:
                    row.update({label+'_shadow_artifact_id':'987654',label+'_shadow_artifact_digest':'a'*64,label+'_shadow_run_attempt':'1'})
                for label in ['native','docs']:
                    if label == 'docs' and abi == 'darwin-arm64': continue
                    row.update({label+'_shadow_artifact_id':'987655',label+'_shadow_artifact_digest':'b'*64,label+'_shadow_run_attempt':'1'})
                if defect and abi=='darwin-arm64':row['ts_'+defect]={'artifact_id':'999999','artifact_digest':'0'*64,'run_attempt':'3'}[defect]
                env['FARM_SUITE_UPLOADS_'+platform]=json.dumps(row)
            for artifact in artifacts.values():artifact['created_at']='2026-10-31T01:03:01Z'
            with tempfile.TemporaryDirectory() as tmp:
                selected=module.current_upload_outputs(env)
                if defect:
                    with self.assertRaises(ValueError):module.collect(transport,run,source,Path(tmp)/'out',upload_outputs=selected)
                else:
                    proof=module.collect(transport,run,source,Path(tmp)/'out',upload_outputs=selected)
                    self.assertEqual(len(proof),6)
                    self.assertEqual({p['runAttempt'] for p in proof.values()},{1})

    def test_workflow_outputs_come_from_fixed_upload_action_and_original_execution(self):
        root=Path(__file__).parents[1]
        primary=(root/'.github/workflows/ci-primary-platform.yaml').read_text()
        caller=(root/'.github/workflows/ci.yaml').read_text()
        for job in ['examples-test','ts-test']:
            block=primary.split('  '+job+':\n',1)[1].split('\n  ts-test:',1)[0]
            self.assertIn('artifact_id: ${{ steps.suite-upload.outputs.artifact-id }}',block)
            self.assertIn('artifact_digest: ${{ steps.suite-upload.outputs.artifact-digest }}',block)
            self.assertIn('run_attempt: ${{ steps.suite-upload-origin.outputs.attempt }}',block)
            self.assertIn('id: suite-upload\n        uses: actions/upload-artifact@v4',block)
            self.assertIn('echo "attempt=$GITHUB_RUN_ATTEMPT" >> "$GITHUB_OUTPUT"',block)
        for platform in ['linux','macos','windows']:
            self.assertEqual(caller.count('FARM_SUITE_UPLOADS_'+platform.upper()+': ${{ toJSON(needs.primary-'+platform+'.outputs) }}'),2)

if __name__=='__main__':unittest.main()
