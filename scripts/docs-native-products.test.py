"""Offline actual graph/archive tests; GitHub API identity is fixture evidence."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location('docs', Path(__file__).with_name('docs-native-products.py'))
docs = importlib.util.module_from_spec(spec); spec.loader.exec_module(docs)


class DocsProducts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = docs.formal.evidence.source_reader(docs.formal.chain.impact.git('rev-parse', 'HEAD').decode().strip())
        cls.wanted = docs.inputs(cls.source, 'darwin-arm64')

    def fixture(self, mutate=None):
        self.abi = 'darwin-arm64'
        self.run = {'id': 10, 'run_attempt': 2, 'path': '.github/workflows/ci.yaml', 'event': 'push',
                    'head_sha': self.source['sha'], 'repository': {'id': 1}, 'head_repository': {'id': 1}}
        self.job = {'steps': [{'name': docs.UPLOAD, 'status': 'completed', 'conclusion': 'success', 'started_at': '1', 'completed_at': '3'}]}
        with tempfile.TemporaryDirectory() as root:
            records=[]
            for row in self.wanted.values():
                body=b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01'+row['owner'].encode()
                file=Path(root,row['file']);file.parent.mkdir(parents=True,exist_ok=True);file.write_bytes(body)
                records.append({**row,'bytes':len(body),'sha256':hashlib.sha256(body).hexdigest()})
            capture={'originBuildSHA':self.source['sha'],'runId':'10','runAttempt':'2','abi':self.abi,'profile':'ci','records':records,
                     'runtimeCapture':{'abi':self.abi,'target':docs.formal.paths.TARGETS[self.abi],'profile':'ci','rustc':'pinned','cargo':'pinned',
                                       'runtime':{'node':'v22.20.0'},'activeToolchain':self.wanted['@farmfe/core']['recipe']['toolchain']+'-aarch64-apple-darwin','errors':[],'unboundBuildVariables':[],'formalFlagsHash':'a'*64}}
            manifest,bodies=docs.produce(self.source,self.abi,capture,Path(root),10,2)
        if mutate:mutate(manifest,bodies)
        stream=io.BytesIO()
        with zipfile.ZipFile(stream,'w') as archive:
            archive.writestr('manifest.json',json.dumps(manifest))
            for name,body in bodies.items():archive.writestr(name,body)
        body=stream.getvalue()
        self.artifact={'id':20,'name':f"docs-native-{self.source['sha']}-{self.abi}-2",'workflow_run':{'id':10},'expired':False,
                       'created_at':'2','size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()}
        return body

    def inspect(self,body):return docs.inspect(body,self.artifact,self.run,self.job,self.source,self.abi)

    def test_full_cohort_bytes_reused(self):
        reused,missing=self.inspect(self.fixture());self.assertEqual(set(reused),set(docs.OWNERS));self.assertEqual(missing,[])

    def test_recipe_mismatch_rebuilds_only_owner(self):
        body=self.fixture(lambda m,b:m['records'][0].update(contentKey='changed'))
        reused,missing=self.inspect(body);self.assertEqual(missing,['@farmfe/core']);self.assertEqual(len(reused),11)

    def test_source_attempt_and_target_cannot_be_forged(self):
        for key,value in [('sourceSHA','a'*40),('sourceTree','b'*40),('attempt',1),('runId',11),('abi','linux-x64-gnu')]:
            with self.subTest(key=key):
                body=self.fixture(lambda m,b:m.update({key:value}))
                with self.assertRaisesRegex(ValueError,'provenance'):self.inspect(body)

    def test_digest_and_member_mutation_rejected(self):
        body=self.fixture();self.artifact['digest']='sha256:'+'f'*64
        with self.assertRaisesRegex(ValueError,'digest'):self.inspect(body)
        body=self.fixture(lambda m,b:b.update({next(iter(b)):b'wrong'}))
        with self.assertRaisesRegex(ValueError,'digest'):self.inspect(body)

    def test_wrong_owner_path_and_upload_rejected(self):
        body=self.fixture(lambda m,b:m['records'][0].update(file='outside.node'))
        with self.assertRaisesRegex(ValueError,'destination'):self.inspect(body)
        body=self.fixture();self.job['steps'][0]['conclusion']='failure'
        with self.assertRaisesRegex(ValueError,'upload'):self.inspect(body)

    def test_missing_expired_and_latest_failed_producer(self):
        self.fixture();self.abi='linux-x64-gnu'
        class Transport:
            def __init__(self,rows):self.rows=rows
            def get(self,path):return {'artifacts':self.rows,'total_count':len(self.rows)}
        with patch.object(docs.execution,'actual_producer',return_value=(self.run,self.job)):
            result=docs.resolve(Transport([]),self.run,self.source,self.abi);self.assertEqual(result[1],docs.OWNERS)
            row={'name':f"docs-native-{self.source['sha']}-{self.abi}-2",'expired':True}
            self.assertEqual(docs.resolve(Transport([row]),self.run,self.source,self.abi)[1],docs.OWNERS)
        for status in ['failure','skipped','cancelled']:
            with patch.object(docs.execution,'actual_producer',side_effect=ValueError('Latest producer '+status)):
                with self.assertRaisesRegex(ValueError,status):docs.resolve(Transport([]),self.run,self.source,self.abi)

    def test_download_disappearance_falls_back_but_denial_and_bad_proof_do_not(self):
        self.fixture(); self.abi='linux-x64-gnu'
        name=f"docs-native-{self.source['sha']}-{self.abi}-2"
        class Transport:
            def __init__(self,error):self.error=error
            def get(self,path):
                if '/artifacts?' in path:return {'artifacts':[{'id':20,'name':name,'expired':False}], 'total_count':1}
                return {'id':20,'expired':False}
            def archive(self,identity):raise self.error
        with patch.object(docs.execution,'actual_producer',return_value=(self.run,self.job)):
            for message in ['GitHub evidence unavailable (HTTP 404)','GitHub artifact read unavailable (HTTP 410)']:
                result=docs.resolve(Transport(ValueError(message)),self.run,self.source,self.abi)
                self.assertEqual(result[1],docs.OWNERS)
            for error in [PermissionError('denied'),ValueError('Docs archive digest differs')]:
                with self.assertRaises(type(error)):docs.resolve(Transport(error),self.run,self.source,self.abi)

    def test_fork_source_build_never_reads_artifacts(self):
        self.fixture(); self.run['head_repository']['id']=2
        result=docs.resolve(None,self.run,self.source,'linux-x64-gnu')
        self.assertEqual(result,({},docs.OWNERS,'fork-source-build'))
        self.run['head_sha']='f'*40
        with self.assertRaisesRegex(ValueError,'source differs'):docs.resolve(None,self.run,self.source,'linux-x64-gnu')

    def test_supported_platform_closure_is_exact(self):
        for abi in docs.PLATFORMS:self.assertEqual(set(docs.inputs(self.source,abi)),set(docs.OWNERS))
        with self.assertRaisesRegex(ValueError,'Unsupported'):docs.inputs(self.source,'win32-arm64-msvc')


if __name__=='__main__':unittest.main()
