# cspell:words sigstore
"""Offline resolver integration tests: real source graph, synthetic API/bytes."""
import copy
import contextlib
import os
import subprocess
import sys
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location('formal', Path(__file__).with_name('formal-native-products.py'))
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
ABI = 'darwin-arm64'


class FormalProducts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = formal.evidence.source_reader(formal.chain.impact.git('rev-parse', 'HEAD').decode().strip())
        cls.products = formal.product_inputs(cls.source, ABI)

    def context(self):
        return {'toolchain': self.products['@farmfe/core']['recipe']['toolchain'], 'rustc': 'pinned rustc',
                'cargo': 'pinned cargo', 'linker': {'version': 'Apple clang'},
                'platform': {'sdk': '27.2', 'host': {'node': 'v22.20.0', 'os': 'darwin', 'arch': 'arm64', 'release': '27.2'}},
                'formalFlagsHash': 'f' * 64}

    def fixture(self, transform=None):
        context = self.context()
        records, bodies = [], {}
        for index, product in enumerate(self.products.values()):
            member = str(index) + '.node'
            body = b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01' + ('synthetic byte fixture for ' + product['owner']).encode()
            bodies[member] = body
            records.append({**copy.deepcopy(product), 'member': member, 'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest()})
        manifest = {'schemaVersion': 2, 'kind': 'main-formal-native-products', 'sourceSHA': self.source['sha'],
                    'sourceTree': self.source['tree'], 'runId': 10, 'attempt': 2, 'abi': ABI, 'context': context, 'records': records}
        if transform: transform(manifest)
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, 'w') as archive:
            archive.writestr('catalog.json', json.dumps(manifest))
            for name, body in bodies.items(): archive.writestr(name, body)
        body = buffer.getvalue()
        run = {'id': 10, 'path': formal.WORKFLOW, 'head_branch': 'main', 'event': 'push', 'status': 'completed',
               'conclusion': 'success', 'repository': {'id': 1}, 'head_repository': {'id': 1}, 'run_attempt': 2, 'head_sha': self.source['sha']}
        artifact = {'id': 20, 'workflow_run': {'id': 10}, 'expired': False, 'name': 'formal-native-' + ABI + '-2',
                    'size_in_bytes': len(body), 'digest': 'sha256:' + hashlib.sha256(body).hexdigest(), 'created_at': '2026-11-01T10:00:01Z'}
        job = {'run_id': 10, 'run_attempt': 2, 'name': 'Build and Upload User Artifacts - ' + ABI, 'status': 'completed', 'conclusion': 'success',
               'steps': [{'name': formal.UPLOAD, 'status': 'completed', 'conclusion': 'success',
                          'started_at': '2026-11-01T10:00:00Z', 'completed_at': '2026-11-01T10:00:02Z'}]}
        desired = {**context, 'repositoryId': 1}
        return body, artifact, run, job, desired

    def inspect(self, values, current=None):
        body, artifact, run, job, desired = values
        return formal.inspect_catalog(body, artifact, run, job, self.source, current or self.source, ABI, desired,
            trusted_output={'artifact_id':str(artifact['id']), 'artifact_digest':artifact['digest'].removeprefix('sha256:'), 'run_attempt':str(run['run_attempt'])})

    def test_unsigned_catalog_cannot_use_upload_time_as_authority(self):
        body, artifact, run, job, desired = self.fixture()
        with self.assertRaisesRegex(ValueError, 'Unsigned formal catalog'):
            formal.inspect_catalog(body, artifact, run, job, self.source, self.source, ABI, desired)
        for step in job['steps']:
            step.pop('started_at'); step.pop('completed_at')
        artifact.pop('created_at')
        self.assertEqual(set(self.inspect((body, artifact, run, job, desired))), set(self.products))

    def test_matching_main_products_reused_without_test_acceptance(self):
        accepted = self.inspect(self.fixture())
        self.assertEqual(set(accepted), set(self.products))
        self.assertTrue(all(value['originSHA'] == self.source['sha'] for value in accepted.values()))

    def test_plugin_source_change_preserves_core_key(self):
        current = copy.deepcopy(self.source)
        source_file = next(file for file in current['files'] if file.startswith('rust-plugins/replace-dirname/src/') and file.endswith('.rs'))
        current['files'][source_file] += b'\n// changed input\n'
        changed = formal.product_inputs(current, ABI)
        self.assertEqual(changed['@farmfe/core']['contentKey'], self.products['@farmfe/core']['contentKey'])
        owner = next(owner for owner, row in changed.items() if row['contentKey'] != self.products[owner]['contentKey'])
        accepted = self.inspect(self.fixture(), current)
        self.assertNotIn(owner, accepted)
        self.assertIn('@farmfe/core', accepted)

    def test_symlink_ancestor_never_creates_outside_directory(self):
        with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as outside:
            Path(root, 'redirect').symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'redirects'):
                formal.workspace_path(root, 'redirect/new/output.node', create_parents=True)
            self.assertFalse(Path(outside, 'new').exists())

    def test_binary_architecture_rejection(self):
        self.assertTrue(formal.native_matches(b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01', ABI))
        self.assertFalse(formal.native_matches(b'\xcf\xfa\xed\xfe\x07\x00\x00\x01', ABI))
        self.assertFalse(formal.native_matches(b'not a binary', ABI))

    def test_create_binding_and_embedded_templates_invalidate_create(self):
        for prefix in ['packages/create-farm/src/', 'crates/create-farm-rs/templates/']:
            current = copy.deepcopy(self.source)
            file = next(file for file in current['files'] if file.startswith(prefix))
            current['files'][file] += b' changed'
            changed = formal.product_inputs(current, ABI)
            self.assertNotEqual(changed['create-farm']['contentKey'], self.products['create-farm']['contentKey'])

    def test_main_sha_is_provenance_not_content_key(self):
        current = {**self.source, 'sha': '1' * 40, 'tree': '2' * 40}
        self.assertEqual(formal.product_inputs(current, ABI), self.products)

    def test_wrong_profile_fingerprint_rejected(self):
        values = self.fixture(lambda manifest: manifest['records'][0]['recipe'].update(profile='ci'))
        with self.assertRaisesRegex(ValueError, 'fingerprint'): self.inspect(values)

    def test_wrong_manifest_abi_rejected(self):
        with self.assertRaisesRegex(ValueError, 'ABI'):
            self.inspect(self.fixture(lambda manifest: manifest.update(abi='darwin-x64')))

    def test_wrong_job_attempt_rejected(self):
        values = self.fixture(); values[3]['run_attempt'] = 1
        with self.assertRaisesRegex(ValueError, 'attempt'): self.inspect(values)

    def test_expired_damaged_fork_or_failed_producer_rejected(self):
        for field, value in [('expired', True), ('digest', 'sha256:' + '0' * 64)]:
            with self.subTest(field=field):
                values = self.fixture(); values[1][field] = value
                with self.assertRaises(ValueError): self.inspect(values)
        for update in [{'head_repository': {'id': 9}}, {'head_branch': 'feature'}]:
            values = self.fixture(); values[2].update(update)
            with self.assertRaises(ValueError): self.inspect(values)

    def test_measured_compiler_mismatch_or_missing_rebuilds(self):
        for field in ['toolchain', 'rustc', 'cargo', 'linker']:
            for missing in [False, True]:
                with self.subTest(field=field, missing=missing):
                    values = self.fixture()
                    if missing: del values[4][field]
                    else: values[4][field] = 'unmatched'
                    with self.assertRaisesRegex(ValueError, 'compiler or linker'): self.inspect(values)

    def test_environment_mismatch_rebuilds(self):
        values = self.fixture(); values[4]['formalFlagsHash'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'flags'): self.inspect(values)

    def test_successful_native_is_independent_of_later_publish_or_other_abi_failure(self):
        for status, conclusion in [('completed','failure'), ('in_progress',None), ('completed','cancelled')]:
            values=self.fixture(); values[2].update(status=status,conclusion=conclusion)
            self.assertEqual(set(self.inspect(values)),set(self.products))
        for conclusion in ['failure','cancelled','skipped']:
            values=self.fixture();values[3]['conclusion']=conclusion
            with self.assertRaisesRegex(ValueError,'Native job failed'):self.inspect(values)

    def resolver_transport(self, values):
        source=self.source; requests=[]
        class Transport:
            def get(self,route):
                requests.append(route)
                if route=='/commits/main':return {'sha':source['sha']}
                if '/workflows/' in route:return {'workflow_runs':[{'id':10,'run_attempt':2}]}
                if route=='/actions/runs/10':return values[2]
                if route.startswith('/compare/'):return {'status':'identical','merge_base_commit':{'sha':source['sha']}}
                raise AssertionError(route)
            def archive(self,identity):raise AssertionError('Unsigned bytes must not be downloaded')
        return Transport(),requests

    def test_unsigned_or_missing_proof_is_a_miss_without_catalog_download(self):
        for reason in ['unsigned','expired','wrong signer','missing proof']:
            transport,requests=self.resolver_transport(self.fixture())
            def deny(*args):raise ValueError(reason)
            accepted,reasons=formal.resolve_products(transport,self.source,ABI,self.context(),
                read_source=lambda sha:self.source,proof_provider=deny)
            self.assertEqual(accepted,{})
            self.assertEqual(reasons,[reason])
            self.assertFalse(any('/attempts/' in route for route in requests))

    def test_new_attempt_never_falls_back_to_old_proof(self):
        values=self.fixture();values[2]['run_attempt']=3
        transport,requests=self.resolver_transport(values)
        def denied(*args):raise AssertionError('Moved attempt must fail before verification')
        accepted,reasons=formal.resolve_products(transport,self.source,ABI,self.context(),proof_provider=denied)
        self.assertEqual(accepted,{})
        self.assertEqual(reasons,['Producer attempt moved during resolution'])

    def test_content_key_hit_across_commit_and_misses_on_shared_inputs(self):
        current=copy.deepcopy(self.source);current['sha']='a'*40
        self.assertEqual(set(self.inspect(self.fixture(),current)),set(self.products))
        for file in ['Cargo.lock','rust-toolchain.toml']:
            current=copy.deepcopy(self.source);current['files'][file]+=b'\n# changed input\n'
            self.assertEqual(self.inspect(self.fixture(),current),{})

    def test_resolver_accumulates_partial_catalogs_with_real_artifact_validation(self):
        # Only API and cryptographic verification are simulated. Discovery,
        # certificate/manifest policy, producer binding and catalog validation
        # all execute production code; no hosted signing is claimed.
        provenance=formal.load('full_native_provenance','release-provenance.py')
        values=[];owners=list(self.products);requests=[];downloaded=[];proofs={};certificates={}
        for index,selected in enumerate([owners[:1],owners[1:]]):
            body,artifact,run,job,desired=self.fixture()
            run['id']=10+index;run['conclusion']='failure'
            run['repository']={'id':507542208,'full_name':'farm-fe/farm'}
            run['head_repository']={'id':507542208}
            desired['repositoryId']=507542208
            job.update(id=30+index,run_id=run['id'],head_sha=self.source['sha'])
            with zipfile.ZipFile(io.BytesIO(body)) as archive:
                manifest=json.loads(archive.read('catalog.json'));manifest['runId']=run['id']
                manifest['records']=[row for row in manifest['records'] if row['owner'] in selected]
                output=io.BytesIO()
                with zipfile.ZipFile(output,'w') as target:
                    target.writestr('catalog.json',json.dumps(manifest))
                    for row in manifest['records']:target.writestr(row['member'],archive.read(row['member']))
            body=output.getvalue()
            artifact.update(id=20+index,workflow_run={'id':run['id']},size_in_bytes=len(body),digest='sha256:'+hashlib.sha256(body).hexdigest())
            values.append((body,artifact,run,job,desired))
            expected=provenance.expected_policy(run,self.source,'native-'+ABI,'c'*40)
            row={'cell':'native','producer':job['name'],'jobId':job['id'],'originAttempt':2,
                 'uploadStep':formal.UPLOAD,'artifactId':artifact['id'],'artifactDigest':artifact['digest'],'artifactName':artifact['name']}
            subject=json.dumps({'schemaVersion':1,'kind':'fixed-workflow-upload-bindings',
                **{key:expected[key] for key in ['repositoryId','sourceSHA','sourceTree','runId','callerWorkflow']},
                'aggregationAttempt':2,'bindings':[row]}).encode()
            repo='https://github.com/farm-fe/farm';signer=repo+'/'+expected['signerWorkflow']+'@'+expected['signerSHA']
            certificate={'issuer':'https://token.actions.githubusercontent.com','subjectAlternativeName':signer,
                'buildSignerURI':signer,'buildSignerDigest':expected['signerSHA'],'runnerEnvironment':'github-hosted',
                'sourceRepositoryURI':repo,'sourceRepositoryIdentifier':'507542208',
                'sourceRepositoryDigest':self.source['sha'],'sourceRepositoryRef':'refs/heads/main',
                'buildConfigURI':repo+'/'+formal.WORKFLOW+'@refs/heads/main','buildConfigDigest':self.source['sha'],
                'buildTrigger':'push','runInvocationURI':repo+'/actions/runs/'+str(run['id'])+'/attempts/2'}
            statement={'_type':'https://in-toto.io/Statement/v1','predicateType':provenance.policy.PREDICATE,
                'subject':[{'name':'ci-bindings.json','digest':{'sha256':hashlib.sha256(subject).hexdigest()}}],
                'predicate':{'schemaVersion':1,'kind':'fixed-workflow-upload-bindings'}}
            certificates[run['id']]=[{'verificationResult':{'signature':{'certificate':certificate},'statement':statement}}]
            archive=io.BytesIO()
            with zipfile.ZipFile(archive,'w') as target:
                target.writestr('ci-bindings.json',subject)
                target.writestr('ci-bindings.sigstore.json',b'explicit offline cryptographic boundary')
            proof_body=archive.getvalue()
            metadata={'id':400+index,'name':'farm-release-bindings-native-'+ABI+'-'+str(run['id'])+'-2',
                'expired':False,'workflow_run':{'id':run['id']},'size_in_bytes':len(proof_body),
                'digest':'sha256:'+hashlib.sha256(proof_body).hexdigest()}
            proofs[run['id']]=(metadata,proof_body)
        source=self.source
        class API:
            def get(self,route):
                requests.append(route)
                if '/workflows/' in route:return {'workflow_runs':[item[2] for item in values]}
                if route=='/commits/main':return {'sha':source['sha']}
                if route.startswith('/compare/'):return {'status':'identical','merge_base_commit':{'sha':source['sha']}}
                for body,artifact,run,job,_ in values:
                    prefix='/actions/runs/'+str(run['id'])
                    if route in [prefix,prefix+'/attempts/2']:return run
                    meta,_=proofs[run['id']]
                    if route.startswith(prefix+'/artifacts?'):return {'total_count':1,'artifacts':[meta]}
                    if route=='/actions/artifacts/'+str(meta['id']):return meta
                    if route.startswith(prefix+'/attempts/2/jobs?'):return {'total_count':1,'jobs':[job]}
                    if route=='/actions/artifacts/'+str(artifact['id']):return artifact
                raise AssertionError(route)
            def archive(self,identity):
                downloaded.append(identity)
                for meta,body in proofs.values():
                    if meta['id']==identity:return body
                return next(item[0] for item in values if item[1]['id']==identity)
        def proof(api,run,source,scope):
            def verified_crypto(subject,bundle,expected):
                self.assertEqual(bundle.read_bytes(),b'explicit offline cryptographic boundary')
                return provenance.policy.validate_verified_manifest(subject.read_bytes(),certificates[run['id']],expected)
            return provenance.verified_run(api,run,source,scope,verify=verified_crypto,signer_sha='c'*40)
        accepted,reasons=formal.resolve_products(API(),self.source,ABI,values[0][4],read_source=lambda sha:self.source,proof_provider=proof)
        self.assertEqual(reasons,[]);self.assertEqual(set(accepted),set(self.products));self.assertEqual(downloaded,[400,20,401,21])
        # Reusing any one successfully signed build does not imply whole-run success.
        self.assertEqual({row['runId'] for row in accepted.values()},{10,11})
        for field in ['buildSignerDigest','sourceRepositoryDigest','runInvocationURI']:
            cert=certificates[11][0]['verificationResult']['signature']['certificate'];original=cert[field]
            cert[field]='wrong'
            accepted,reasons=formal.resolve_products(API(),self.source,ABI,values[0][4],read_source=lambda sha:self.source,proof_provider=proof)
            self.assertEqual(set(accepted),set(owners[:1]));self.assertIn('Verified certificate identity differs or is incomplete',reasons)
            cert[field]=original
        values[1][1]['expired']=True
        accepted,reasons=formal.resolve_products(API(),self.source,ABI,values[0][4],read_source=lambda sha:self.source,proof_provider=proof)
        self.assertEqual(set(accepted),set(owners[:1]));self.assertIn('Signed artifact identity differs',reasons)

    def captured_windows(self, abi):
        # Execute the actual JS collector; tool output is an explicit fixture,
        # not a claim that this host executed Windows tools.
        program = """import {captureRuntime} from './scripts/ci-native-capture.mjs';
const [abi,target]=process.argv.slice(1);
console.log(JSON.stringify(captureRuntime({platform:'win32',env:{FARM_BUILD_ABI:abi,FARM_BUILD_TARGET:target,FARM_BUILD_PROFILE:'release'},
run:exe=>({status:0,stdout:exe==='rustup'?'nightly-2026-04-10-x86_64-pc-windows-msvc':exe+' measured fixture'})})));"""
        runtime = json.loads(subprocess.check_output(['node', '--input-type=module', '-e', program, abi, formal.paths.TARGETS[abi]], cwd=Path(__file__).resolve().parent.parent))
        runtime['runtime']['node'] = 'v22.20.0'  # Explicit simulated runner, not host measurement.
        return runtime

    def test_windows_collector_null_reaches_explicit_unavailable_producer(self):
        for abi in ['win32-arm64-msvc', 'win32-ia32-msvc']:
            runtime = self.captured_windows(abi)
            self.assertIsNone(runtime['linker']); self.assertFalse(runtime['reuseAuthorized'])
            with tempfile.TemporaryDirectory() as root:
                root = Path(root); capture = root / 'capture.json'; output = root / 'catalog'; outputs = root / 'outputs'
                capture.write_text(json.dumps({'runtimeCapture': runtime, 'records': []}))
                env = {'GITHUB_SHA': self.source['sha'], 'GITHUB_REPOSITORY': 'fixture/farm', 'GITHUB_WORKFLOW_REF': 'fixture/farm/' + formal.WORKFLOW + '@refs/heads/main', 'GITHUB_REF': 'refs/heads/main', 'FARM_BUILD_PROFILE': 'release', 'GITHUB_OUTPUT': str(outputs)}
                with patch.dict(os.environ, env), patch.object(sys, 'argv', ['formal-native-products.py', 'produce', '--abi', abi, '--capture', str(capture), '--output', str(output)]), patch.object(formal.evidence, 'source_reader', return_value=self.source), patch.object(formal, 'create_catalog') as create, contextlib.redirect_stdout(io.StringIO()) as stdout:
                    formal.main()
                result = json.loads(stdout.getvalue())
                self.assertFalse(result['catalogAvailable']); self.assertEqual(result['consumerAction'], 'build-current')
                self.assertEqual(result['reason'], 'Unmeasured effective linker')
                self.assertEqual(outputs.read_text(), 'catalog_available=false\n'); self.assertFalse(output.exists()); create.assert_not_called()

    def test_missing_null_and_malformed_measured_objects_are_not_reuse_evidence(self):
        abi = 'win32-arm64-msvc'; runtime = self.captured_windows(abi)
        for value in [None, [], '', 1, {}, {'version': None}, {'version': []}, {'version': ''}, {'version': ' '}]:
            with self.subTest(linker=value), self.assertRaisesRegex(ValueError, 'Unmeasured effective linker'):
                formal.measured_context(self.source, {'runtimeCapture': {**runtime, 'linker': value}}, abi)
        absent = dict(runtime); absent.pop('linker')
        with self.assertRaisesRegex(ValueError, 'Unmeasured effective linker'): formal.measured_context(self.source, {'runtimeCapture': absent}, abi)
        for field in ['runtime', 'activeToolchain']:
            with self.subTest(field=field), self.assertRaises(ValueError):
                formal.measured_context(self.source, {'runtimeCapture': {**runtime, field: None}}, abi)
        with self.assertRaises(ValueError): formal.measured_context(self.source, {'runtimeCapture': None}, abi)

    def test_measured_windows_fixture_seals_only_bound_unchanged_native_bytes(self):
        abi = 'win32-arm64-msvc'; runtime = self.captured_windows(abi)
        # Positive measurement fixture, not inferred from a missing linker.
        runtime.update(linker={'version': 'explicit measured linker fixture'}, platformSdk='explicit measured SDK fixture')
        wanted = formal.product_inputs(self.source, abi)['@farmfe/core']
        with tempfile.TemporaryDirectory() as root:
            root = Path(root); file = root / wanted['file']; file.parent.mkdir(parents=True)
            body = b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\0\0' + (0xaa64).to_bytes(2, 'little'); file.write_bytes(body)
            capture = {'runtimeCapture': runtime, 'profile': 'release', 'originBuildSHA': self.source['sha'], 'runId': 10, 'runAttempt': 2,
                       'records': [{**wanted, 'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest()}]}
            catalog, bodies = formal.create_catalog(self.source, capture, abi, 10, 2, root)
            self.assertEqual(catalog['records'][0]['sha256'], hashlib.sha256(body).hexdigest()); self.assertEqual(list(bodies.values()), [body])
            with self.assertRaisesRegex(ValueError, 'identity'): formal.create_catalog(self.source, capture, abi, 10, 3, root)
            file.write_bytes(body + b'changed')
            with self.assertRaisesRegex(ValueError, 'bytes changed'): formal.create_catalog(self.source, capture, abi, 10, 2, root)


if __name__ == '__main__': unittest.main()
