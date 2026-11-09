"""Real receipt ZIPs and fixed workflow interfaces; no hosted identity claim."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import io
import json
import subprocess
from pathlib import Path
import unittest
import zipfile
spec=importlib.util.spec_from_file_location('receipts',Path(__file__).with_name('release-consumer-bindings.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


class Contracts(unittest.TestCase):
    def fixture(self,kind='candidate',platform='macos-latest'):
        manifest={'sourceSHA':'a'*40,'sourceTree':'b'*40,'producerRunId':42,'producerAttempt':1}
        digest=hashlib.sha256(json.dumps(manifest,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        binding={'manifestHash':digest,**manifest,'consumerRunId':42,'consumerAttempt':1}
        report={'status':'passed','scratchRemoved':True,'cleanupConfirmed':True}
        if kind=='candidate':report.update(releaseBinding=binding,platform=m.ABI[platform])
        else:report.update(publicationBinding={**binding,'platform':platform},publicRegistry=True,workspacePatches=False,
                           stages=['npm-install','esm-cjs','types','docs-build','sdk'])
        state={'report':report}
        def archive():
            output=io.BytesIO()
            with zipfile.ZipFile(output,'w') as z:z.writestr('result.json',json.dumps(state['report']))
            return output.getvalue()
        body=archive();definition=m.role(kind,platform)
        artifact={'id':100,'name':definition['artifactName'].format(sourceSHA=manifest['sourceSHA'],originAttempt=1),
                  'workflow_run':{'id':42},'expired':False,'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
        run={'id':42,'run_attempt':2,'path':'.github/workflows/incremental-release.yaml','head_sha':manifest['sourceSHA'],
             'event':'push','head_branch':'main','repository':{'id':507542208},'head_repository':{'id':507542208}}
        jobs={i:[{'id':200+i,'name':'Fixed consumer / '+definition['producer'],'run_id':42,'run_attempt':i,
                   'head_sha':run['head_sha'],'status':'completed','conclusion':'success',
                   'steps':[{'name':definition['uploadStep'],'status':'completed','conclusion':'success'}]}] for i in [1,2]}
        class API:
            def get(self,path):
                if path.startswith('/actions/artifacts/'):return artifact
                attempt=int(path.split('/attempts/')[1].split('/')[0])
                if '/jobs?' in path:return {'jobs':jobs[attempt]}
                return {**run,'run_attempt':attempt}
            def archive(self,identity):return archive()
        output={'artifact_id':'100','artifact_digest':artifact['digest'][7:],'run_attempt':'1'}
        return API(),run,manifest,kind,platform,output,jobs,artifact,state

    def test_fixed_workflow_outputs_are_distinct_for_all_three_platforms(self):
        row={label+'_'+field:(str(100+i) if field=='artifact_id' else 'a'*64 if field=='artifact_digest' else '1')
             for i,label in enumerate(m.PLATFORMS) for field in m.FIELDS}
        parsed=m.outputs(json.dumps(row));self.assertEqual(set(parsed),set(m.ABI))
        for path in ['release-candidate-consumers.yaml','release-public-consumers.yaml']:
            # Historical transactions retain their original fixed consumer interface.
            source=subprocess.check_output(['git','show',
                'c3c99547eaba6624c5fce482db92fbce5995ef37:.github/workflows/'+path],
                cwd=Path(__file__).parents[1],text=True)
            self.assertNotIn('matrix:',source)
            for label in m.PLATFORMS:
                for field in m.FIELDS:self.assertIn('${{ jobs.'+label+'.outputs.'+field+' }}',source)
        for defect in ['missing','unknown','duplicate-platform','attempt','digest','duplicate-json']:
            bad=copy.deepcopy(row)
            if defect=='missing':del bad['windows_artifact_id']
            if defect=='unknown':bad['extra']='1'
            if defect=='duplicate-platform':bad['macos_artifact_id']=bad['linux_artifact_id']
            if defect=='attempt':bad['linux_run_attempt']='0'
            if defect=='digest':bad['linux_artifact_digest']='SHA256:bad'
            value=json.dumps(bad)
            if defect=='duplicate-json':value=value[:-1]+',"linux_artifact_id":"100"}'
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.outputs(value)

    def test_real_bytes_bind_to_original_attempt_despite_changed_carried_job_id_and_clocks(self):
        for kind in ['candidate','public']:
            for platform in m.ABI:
                api,run,manifest,kind,platform,output,jobs,artifact,_=self.fixture(kind,platform)
                artifact['created_at']='2099-invalid'
                for rows in jobs.values():
                    rows[0].update(started_at=None,completed_at='1900')
                    rows[0]['steps'][0].update(started_at='arbitrary',completed_at=None)
                report,meta,origin,job=m.receipt(api,run,manifest,kind,platform,output)
                self.assertEqual(origin['run_attempt'],1);self.assertEqual(job['id'],201)
                self.assertEqual(meta['id'],100);self.assertEqual(report['status'],'passed')

    def test_legacy_receipts_reject_byte_only_reports_even_with_valid_upload_digest(self):
        for kind in ['candidate','public']:
            api,run,manifest,kind,platform,output,_,artifact,state=self.fixture(kind)
            state['report']['kind']='release-byte-integrity-v1'
            body=api.archive(artifact['id'])
            artifact.update(size_in_bytes=len(body),digest='sha256:'+hashlib.sha256(body).hexdigest())
            output['artifact_digest']=artifact['digest'][7:]
            with self.subTest(kind=kind),self.assertRaisesRegex(ValueError,'Legacy consumer receipt'):
                m.receipt(api,run,manifest,kind,platform,output)

    def test_new_manifest_cannot_select_legacy_consumer_policy(self):
        for kind in ['candidate','public']:
            api,run,manifest,kind,platform,output,*_=self.fixture(kind)
            manifest['releaseValidation']='archive-integrity-v1'
            with self.subTest(kind=kind),self.assertRaisesRegex(ValueError,'do not use legacy'):
                m.receipt(api,run,manifest,kind,platform,output)

    def test_missing_or_forged_provenance_never_falls_back_to_old_success(self):
        for defect in ['missing-output','id','digest','attempt','platform','source','run','job-sha','job-duplicate',
                       'latest-failure','latest-cancelled','latest-running','missing-upload','upload-failure','bytes']:
            api,run,manifest,kind,platform,output,jobs,artifact,state=self.fixture()
            if defect=='missing-output':output=None
            if defect=='id':output['artifact_id']='101'
            if defect=='digest':output['artifact_digest']='0'*64
            if defect=='attempt':output['run_attempt']='3'
            if defect=='platform':platform='windows-latest'
            if defect=='source':manifest['sourceSHA']='c'*40
            if defect=='run':artifact['workflow_run']['id']=43
            if defect=='job-sha':jobs[1][0]['head_sha']='d'*40
            if defect=='job-duplicate':jobs[1].append(copy.deepcopy(jobs[1][0]))
            if defect.startswith('latest-'):
                jobs[2][0]['conclusion']=defect.removeprefix('latest-')
                if defect=='latest-running':jobs[2][0]['status']='in_progress'
            if defect=='missing-upload':jobs[1][0]['steps']=[]
            if defect=='upload-failure':jobs[1][0]['steps'][0]['conclusion']='failure'
            if defect=='bytes':state['report']['status']='failed'
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.receipt(api,run,manifest,kind,platform,output)


if __name__=='__main__':unittest.main(verbosity=2)
