"""Current Actions output bindings are independent of runner/server clock order."""
import copy
import importlib.util
import json
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
