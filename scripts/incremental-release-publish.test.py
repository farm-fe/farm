# cspell:words GLOBALCONFIG
# cspell:words readback USERCONFIG ENEEDAUTH EOTP mnpm
"""Pure HTTP/Release finalization contracts; no real requests or registry writes."""
# cspell:words importlib stopall finalizer postpublish nonpublishing nonbaseline
import copy
import base64
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading
from types import SimpleNamespace
import hashlib
import importlib.util
import io
import json
import re
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest import mock
import urllib.error
import urllib.parse

spec = importlib.util.spec_from_file_location("release_publish_contract", Path(__file__).with_name("incremental-release-publish.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


class ReleaseAPI:
    def __init__(self):
        self.ref, self.release, self.writes = None, None, []

    def request(self, method, route, data=None, **options):
        if method == "GET":
            return self.ref if route.startswith('/git/') else self.release
        self.writes.append((route, data))
        if route == '/git/refs':
            self.ref = {"object": {"type": "commit", "sha": data["sha"]}}
        elif route == '/releases':
            self.release = {"id": 5, **data}
        return {}


class Contracts(unittest.TestCase):
    def setUp(self):
        receipt = '<!-- mock-verified-checkpoint -->'
        self.checkpoint = mock.patch.object(module.entry, 'release_checkpoint', return_value=receipt).start()
        self.addCleanup(mock.patch.stopall)
        mock.patch.object(module, 'verify_release_checkpoint', side_effect=lambda api, manifest, release: module.check(receipt in release['body'], 'Persistent checkpoint missing')).start()

    def fixture(self):
        return ReleaseAPI(), {"sourceSHA": "a" * 40, "tag": "v2.0.0"}

    def test_tag_and_release_are_read_back_and_retry_is_idempotent(self):
        api, manifest = self.fixture()
        first = module.confirm_release(api, manifest, {})
        self.assertTrue(first["verified"]); self.assertEqual(len(api.writes), 2)
        self.assertEqual(module.confirm_release(api, manifest, {}), first)
        self.assertEqual(len(api.writes), 2)

    def test_existing_different_tag_source_blocks_all_writes(self):
        api, manifest = self.fixture(); api.ref = {"object": {"type": "commit", "sha": "b" * 40}}
        with self.assertRaisesRegex(ValueError, "tag source differs"):
            module.confirm_release(api, manifest, {})
        self.assertEqual(api.writes, [])

    def test_release_failure_after_tag_creation_can_retry_without_moving_tag(self):
        api, manifest = self.fixture(); original = api.request
        def fail(method, route, data=None, **options):
            if method == 'POST' and route == '/releases': raise OSError('Simulated release failure')
            return original(method, route, data, **options)
        api.request = fail
        with self.assertRaises(OSError): module.confirm_release(api, manifest, {})
        self.assertIsNotNone(api.ref); self.assertIsNone(api.release)
        api.request = original
        module.confirm_release(api, manifest, {})
        self.assertEqual([route for route, data in api.writes], ['/git/refs', '/releases'])

    def test_different_release_body_cannot_be_reused(self):
        api, manifest = self.fixture(); module.confirm_release(api, manifest, {})
        api.release['body'] = 'different provenance'
        with self.assertRaisesRegex(ValueError, "Release provenance"):
            module.confirm_release(api, manifest, {})

    def test_release_response_loss_preserves_original_checkpoint_on_retry(self):
        api, manifest = self.fixture(); original = api.request
        def lose_response(method, route, data=None, **options):
            value = original(method, route, data, **options)
            if method == 'POST' and route == '/releases': raise OSError('response lost')
            return value
        api.request = lose_response
        with self.assertRaises(OSError): module.confirm_release(api, manifest, {})
        api.request = original
        module.confirm_release(api, manifest, {'laterAttempt': True})
        self.assertEqual(self.checkpoint.call_count, 1)
        self.assertEqual(len(api.writes), 2)

    def test_existing_release_without_durable_receipt_cannot_advance_baseline(self):
        api, manifest = self.fixture()
        module.confirm_release(api, manifest, {})
        api.release['body'] = '<!-- farm-release-manifest:' + module.state.digest(manifest) + ' -->'
        with self.assertRaisesRegex(ValueError, 'Persistent checkpoint missing'):
            module.confirm_release(api, manifest, {})

    def test_real_entry_checkpoint_finalizer_and_persistent_reader_share_one_receipt(self):
        mock.patch.stopall()
        fixture_module = module.entry.load('finalizer_baseline_fixture', 'incremental-release-baseline.test.py')
        fixture = fixture_module.Contracts(); fixture.setUp()
        api = ReleaseAPI(); api.reader = mock.Mock(); api.reader.get.return_value = {'id': 1}
        original = api.request
        def request(method, route, data=None, **options):
            result = original(method, route, data, **options)
            if method == 'POST' and route == '/releases':
                api.release['author'] = {'login': 'github-actions[bot]', 'type': 'Bot'}
            return result
        api.request = request
        manifest = fixture.manifest
        proof = {'verified': True, 'manifestHash': module.state.digest(manifest),
                 'sourceSHA': fixture.source['sha'], 'sourceTree': fixture.source['tree']}
        with mock.patch.dict(module.os.environ, {'GITHUB_JOB': 'finalize'}), \
             mock.patch.object(module.entry, 'actions_transport', return_value=api.reader), \
             mock.patch.object(module.entry.evidence, 'source_reader', return_value=fixture.source):
            module.confirm_release(api, manifest, proof)
            decoded = module.verify_release_checkpoint(api, manifest, api.release)
            self.assertEqual(set(decoded['packages']), {row['kind'] + ':' + row['name'] for row in manifest['packages']})
            module.confirm_release(api, manifest, {**proof, 'laterAttempt': True})
            self.assertEqual(len(api.writes), 2)

    def test_final_cas_lost_response_rechecks_real_checkpoint_before_already_complete(self):
        mock.patch.stopall()
        fixtures = module.entry.load('lost_cas_checkpoint_fixture', 'incremental-release-checkpoint.test.py')
        fixture = fixtures.Contracts(); fixture.setUp()
        shared = module.entry.load('lost_cas_state_fixture', 'incremental-publication-state.test.py')
        api = ReleaseAPI(); api.reader = mock.Mock(); api.reader.get.return_value = {'id': 1}
        api.ref = {'object': {'type': 'commit', 'sha': fixture.source['sha']}}
        api.release = {'id': 5, **copy.deepcopy(fixture.release)}
        manifest = fixture.manifest; transaction = module.state.digest(manifest)
        store = module.state.GitRefStore(shared.GitAPI())
        complete = {'baseline': transaction, 'active': None, 'completed': {transaction: {
            'manifest': manifest, 'proof': {'manifestHash': transaction},
            'release': {'manifestHash': transaction, 'sourceSHA': manifest['sourceSHA']}}}}
        # Simulate a successful final durable CAS whose acknowledgement is lost.
        def final_cas():
            store.swap(None, complete)
            raise OSError('final CAS response lost')
        with self.assertRaisesRegex(OSError, 'final CAS response lost'): final_cas()
        preflight={'prepublicationVerified':True,'manifestHash':transaction,'sourceSHA':manifest['sourceSHA'],'sourceTree':manifest['sourceTree']}
        registry=mock.Mock()
        registry.verify.side_effect=lambda records:{key:row['archiveSha256'] for key,row in records.items()}
        owner = {'runId': 10, 'attempt': 2}
        valid_body = api.release['body']
        with mock.patch.object(module.entry.evidence, 'source_reader', return_value=fixture.source):
            for mutation in ['missing', 'corrupt', 'valid']:
                with self.subTest(checkpoint=mutation):
                    api.release['body'] = valid_body
                    if mutation == 'missing': api.release['body'] = '<!-- farm-release-manifest:' + transaction + ' -->'
                    if mutation == 'corrupt': api.release['body'] = valid_body.replace('farm-release-checkpoint-v1:', 'farm-release-checkpoint-v1:' + '0')
                    retry = module.state.Publication(store, manifest, owner, lambda value: True)
                    verify = lambda receipt: module.completed_publication_receipt(api, manifest, preflight, registry=registry)
                    if mutation == 'valid':
                        retry.recover(lambda previous: True, verify)
                        self.assertTrue(retry.completed)
                    else:
                        with self.assertRaisesRegex(ValueError, 'checkpoint missing or ambiguous'):
                            retry.recover(lambda previous: True, verify)
                        self.assertFalse(retry.completed)
                    self.assertEqual(api.writes, [])
            self.assertEqual(registry.verify.call_count, 1)
            registry.verify.return_value={}
            registry.verify.side_effect=None
            with self.assertRaisesRegex(ValueError,'registry bytes'):
                module.completed_publication_receipt(api,manifest,preflight,registry=registry)
            with self.assertRaisesRegex(ValueError,'current frozen-byte preflight'):
                module.completed_publication_receipt(api,manifest,{**preflight,'manifestHash':'0'*64},registry=registry)
            self.assertEqual(api.writes, [])

    def environment(self):
        return {"GITHUB_ACTIONS": "true", "GITHUB_REF": "refs/heads/main", "GITHUB_EVENT_NAME": "workflow_dispatch",
                "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_WORKFLOW_REF": "farm-fe/farm/" + module.entry.WORKFLOW + "@refs/heads/main",
                "GITHUB_JOB": "publisher", "FARM_EXPLICIT_PUBLICATION": "true", "GH_TOKEN": "sentinel-secret"}

    def test_publication_is_not_authorized_by_a_caller_role_alone(self):
        for key, value in [('GITHUB_EVENT_NAME','pull_request'),('GITHUB_JOB','plan'),('FARM_EXPLICIT_PUBLICATION','false')]:
            env = {**self.environment(), key: value}
            with self.assertRaises(ValueError): module.GitHubWriter(env, 'upload')

    def test_writer_does_not_allow_other_repositories_or_release_routes_in_upload(self):
        writer = module.GitHubWriter(self.environment(), 'upload', opener=mock.Mock())
        for route in ['https://other.invalid/token', '/../../other', '/releases', '/git/refs/heads/main']:
            with self.assertRaisesRegex(ValueError, 'Unexpected publication API route'):
                writer.request('POST', route, {})
        writer.opener.open.assert_not_called()

    def test_only_exact_missing_ref_404_is_optional_and_errors_hide_tokens(self):
        opener = mock.Mock()
        opener.open.side_effect = urllib.error.HTTPError('https://api.github.com',403,'sentinel-secret',{},None)
        writer = module.GitHubWriter(self.environment(), 'upload', opener=opener)
        with self.assertRaises(ValueError) as context:
            writer.request('GET','/git/ref/heads/farm-release-state',missing=True)
        self.assertNotIn('sentinel-secret', str(context.exception))
        opener.open.side_effect = urllib.error.HTTPError('https://api.github.com',404,'not found',{},None)
        self.assertIsNone(writer.request('GET','/git/ref/heads/farm-release-state',missing=True))
        with self.assertRaises(ValueError): writer.request('GET','/git/commits/'+'a'*40,missing=True)

    def registry(self, metadata, bodies):
        reader=mock.MagicMock();reader.json.side_effect=metadata;reader.archive.side_effect=bodies
        with mock.patch.object(module.entry.evidence.registry_module,'RegistryReadOnly',return_value=reader):
            writer=module.RegistryWriter({'GITHUB_ACTIONS': 'true', 'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://oidc.invalid/', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'oidc-fixture'},'.')
        writer.validate_archive = mock.Mock()  # synthetic bytes in registry-model tests
        self.addCleanup(writer.close)
        return writer,reader

    def metadata(self, checksum):
        return {'name':'a','version':'2.0.0','dist':{'tarball':'https://registry.npmjs.org/a/-/a-2.0.0.tgz','integrity':'sha512-'+checksum}}

    def test_registry_byte_proof_is_bound_to_the_exact_metadata_response(self):
        writer,reader=self.registry([self.metadata('A'),self.metadata('B')],[{'sha256':'d'*64,'sha512':'A'},{'sha256':'d'*64,'sha512':'B'}])
        row={'kind':'npm','name':'a','version':'2.0.0','archiveSha256':'d'*64}
        self.assertEqual(writer.lookup(row),'d'*64);self.assertEqual(writer.lookup(row),'d'*64)
        self.assertEqual(reader.json.call_count,2);self.assertEqual(reader.archive.call_count,2);reader.verify.assert_not_called()

    def test_metadata_a_cannot_cache_a_proof_of_bytes_b(self):
        writer,reader=self.registry([self.metadata('A')],[{'sha256':'d'*64,'sha512':'B'}])
        with self.assertRaisesRegex(ValueError,'exact metadata'):
            writer.lookup({'kind':'npm','name':'a','version':'2.0.0','archiveSha256':'d'*64})
        reader.verify.assert_not_called()

    def test_same_metadata_still_rechecks_public_archive_bytes_on_resume(self):
        writer,reader=self.registry([self.metadata('A'),self.metadata('A')],[{'sha256':'d'*64,'sha512':'A'},{'sha256':'e'*64,'sha512':'A'}])
        row={'kind':'npm','name':'a','version':'2.0.0','archiveSha256':'d'*64}
        writer.lookup(row)
        with self.assertRaisesRegex(ValueError,'different bytes'):writer.lookup(row)
        self.assertEqual(reader.archive.call_count,2)

    def test_terminal_owner_requires_same_run_earlier_attempt_and_exact_source(self):
        source={'sha':'a'*40};current={'runId':7,'attempt':2};owner={'runId':7,'attempt':1,'repositoryId':5}
        run={'id':7,'run_attempt':1,'status':'completed','path':module.entry.WORKFLOW,'head_sha':source['sha'],
             'event':'workflow_dispatch','head_branch':'main','repository':{'id':5},'head_repository':{'id':5}}
        reader=mock.Mock();reader.get.return_value=run
        self.assertTrue(module.terminal_owner(reader,current,source,owner))
        run['status']='in_progress';self.assertFalse(module.terminal_owner(reader,current,source,owner))
        with self.assertRaises(ValueError):module.terminal_owner(reader,current,source,{**owner,'runId':8})

    def recovery_fixture(self, producer_attempt=2, current_attempt=3, candidate_attempts=None):
        spec=importlib.util.spec_from_file_location('shared_release_fixture',Path(__file__).with_name('incremental-release-entry.test.py'))
        shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
        fixture=shared.Contracts();fixture.setUp()
        fixture.repository['id']=507542208
        fixture.run.update(event='push',repository={'id':507542208,'full_name':'farm-fe/farm'},head_repository={'id':507542208})
        for job in fixture.jobs:job['head_sha']=fixture.source['sha']
        fixture.manifest['producerAttempt']=producer_attempt
        fixture.manifest['tag']='v1.0.0'
        transaction=module.state.digest(fixture.manifest)
        fixture.seal['manifestHash']=fixture.public['manifestHash']=transaction
        original={**fixture.run,'run_attempt':producer_attempt,'conclusion':'failure'}
        current={**fixture.run,'run_attempt':current_attempt,'status':'in_progress','conclusion':None}
        runs={attempt:{**original,'run_attempt':attempt} for attempt in range(producer_attempt,current_attempt+1)}
        runs[producer_attempt]=original;runs[current_attempt]=current
        fixture.jobs_by_attempt={attempt:[] for attempt in runs}
        base_jobs=[job for job in fixture.jobs if not job['name'].startswith('Independent release candidate consumer (')]
        for job in base_jobs:job.update({'run_id':10,'run_attempt':producer_attempt})
        fixture.jobs_by_attempt[producer_attempt].extend(base_jobs)
        fixture.jobs_by_attempt[current_attempt].extend([{**job,'run_attempt':current_attempt} for job in base_jobs if job['name']==module.entry.CONSUMER])
        choices=candidate_attempts or {platform:producer_attempt for platform in module.PUBLIC_PLATFORMS}
        metadata={};bodies={};candidate_jobs=[]
        def archive(number,name,value):
            output=io.BytesIO()
            with zipfile.ZipFile(output,'w') as file:file.writestr('result.json' if name.startswith('farm-release-candidate-consumer-') else 'ledger.json',json.dumps(value))
            body=output.getvalue();bodies[number]=body
            metadata[number]={'id':number,'name':name,'workflow_run':{'id':10},'expired':False,'size_in_bytes':len(body),
                              'digest':'sha256:'+hashlib.sha256(body).hexdigest(),'created_at':'2026-01-01T00:00:30Z'}
        expected={'ubuntu-24.04':('linux','x64','linux-x64-gnu'),'macos-latest':('darwin','arm64','darwin-arm64'),'windows-latest':('win32','x64','win32-x64-msvc')}
        for index,platform in enumerate(module.PUBLIC_PLATFORMS,200):
            attempt=choices[platform];os_name,arch,abi=expected[platform]
            report={'status':'passed','scratchRemoved':True,'cleanupConfirmed':True,'platform':{'os':os_name,'arch':arch,'abi':abi},
                    'releaseBinding':{'manifestHash':transaction,'sourceSHA':fixture.source['sha'],'sourceTree':fixture.source['tree'],
                    'producerRunId':10,'producerAttempt':producer_attempt,'consumerRunId':10,'consumerAttempt':attempt}}
            archive(index,'farm-release-candidate-consumer-'+fixture.source['sha']+'-'+platform+'-'+str(attempt),report)
            job={'id':index+100,'run_id':10,'run_attempt':attempt,'name':'Independent release candidate consumer ('+platform+')',
                 'status':'completed','conclusion':'success','steps':[{'name':'Upload independent release candidate acceptance',
                 'status':'completed','conclusion':'success','started_at':'2026-01-01T00:00:00Z','completed_at':'2026-01-01T00:01:00Z'}]}
            job['head_sha']=fixture.source['sha']
            fixture.jobs_by_attempt[attempt].append(job);candidate_jobs.append(job)
            for carried in range(attempt+1,current_attempt+1):
                fixture.jobs_by_attempt[carried].append({**job,'id':job['id']+carried*1000,'run_attempt':carried})
        fixture.jobs=base_jobs+candidate_jobs
        fixture.candidate_outputs={platform:{'artifact_id':str(index),'artifact_digest':metadata[index]['digest'][7:],
                                           'run_attempt':str(choices[platform])} for index,platform in enumerate(module.PUBLIC_PLATFORMS,200)}
        archive(62,'farm-release-candidates-'+fixture.source['sha']+'-'+str(producer_attempt),{'fixture':'candidate bytes'})
        next(job for job in base_jobs if job['name']==module.entry.PRODUCER)['steps'].append({'name':'Freeze independent release candidates','status':'completed','conclusion':'success'})
        metadata[800]={'id':800,'name':'farm-release-bindings-frozen-10-'+str(producer_attempt)}
        archive(61,'farm-release-seal-'+fixture.source['sha']+'-'+str(producer_attempt),fixture.seal)
        archive(60,'farm-release-public-acceptance-'+fixture.source['sha']+'-'+str(current_attempt),fixture.public)
        public_outputs={};public_selections=[]
        for index,platform in enumerate(module.PUBLIC_PLATFORMS,400):
            report={'status':'passed','publicRegistry':True,'workspacePatches':False,'cleanupConfirmed':True,
                    'stages':['npm-install','esm-cjs','types','docs-build','sdk'],
                    'publicationBinding':module.report_binding(fixture.manifest,10,current_attempt,platform)}
            archive(index,'release-public-'+platform+'-'+str(current_attempt),report)
            # Public receipts use result.json, independently of seal ledgers.
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,'w') as z:z.writestr('result.json',json.dumps(report))
            bodies[index]=stream.getvalue();metadata[index].update(size_in_bytes=len(bodies[index]),digest='sha256:'+hashlib.sha256(bodies[index]).hexdigest())
            job={'id':index+100,'run_id':10,'run_attempt':current_attempt,'head_sha':fixture.source['sha'],
                 'name':'Independent public registry consumer ('+platform+')','status':'completed','conclusion':'success',
                 'steps':[{'name':module.PUBLIC_UPLOAD,'status':'completed','conclusion':'success'}]}
            fixture.jobs_by_attempt[current_attempt].append(job)
            public_outputs[platform]={'artifact_id':str(index),'artifact_digest':metadata[index]['digest'][7:],'run_attempt':str(current_attempt)}
            public_selections.append({'platform':platform,'runId':10,'attempt':current_attempt,'artifactId':index,
                'artifactDigest':metadata[index]['digest'],'jobId':job['id'],'manifestHash':transaction})
        fixture.public['consumerReports']=sorted(public_selections,key=lambda row:row['platform'])
        archive(60,'farm-release-public-acceptance-'+fixture.source['sha']+'-'+str(current_attempt),fixture.public)
        def signed_provider(client,observed,source,scope):
            # Explicit offline cryptographic boundary; real ZIP/API binding follows.
            proof=module.load('fixture_public_provenance','release-provenance.py')
            expected=proof.expected_policy(observed,source,scope)
            bindings={}
            for cell,role in expected['cells'].items():
                if cell in ['seal','candidates']:identity=61 if cell=='seal' else 62;origin=producer_attempt
                elif cell=='public':identity=60;origin=current_attempt
                elif cell.startswith('candidate:'):
                    output=fixture.candidate_outputs[cell.split(':',1)[1]];identity=int(output['artifact_id']);origin=int(output['run_attempt'])
                else:
                    output=public_outputs[cell.split(':',1)[1]];identity=int(output['artifact_id']);origin=int(output['run_attempt'])
                job=next(row for row in fixture.jobs_by_attempt[origin] if row['name']==role['producer'])
                meta=metadata[identity]
                bindings[cell]={'cell':cell,**{key:role[key] for key in ['producer','uploadStep']},'jobId':job['id'],
                    'originAttempt':origin,'artifactId':identity,'artifactName':meta['name'],'artifactDigest':meta['digest']}
            return expected,bindings
        fixture.proof_provider=signed_provider
        def get(route):
            if route=='':return fixture.repository
            if route=='/branches/main':return {'commit':{'sha':fixture.source['sha']}}
            if route=='/actions/runs/10':return current
            match=re.fullmatch(r'/actions/runs/10/attempts/([1-9][0-9]*)(/jobs\?per_page=100&page=1)?',route)
            if match:
                attempt=int(match[1]);return {'jobs':fixture.jobs_by_attempt[attempt]} if match[2] else runs[attempt]
            if route.startswith('/actions/runs/10/artifacts?'):
                query=urllib.parse.parse_qs(urllib.parse.urlsplit(route).query)
                rows=list(metadata.values())
                if 'name' in query:rows=[row for row in rows if row['name']==query['name'][0]]
                return {'total_count':len(rows),'artifacts':rows}
            if route.startswith('/actions/artifacts/'):return metadata[int(route.rsplit('/',1)[1])]
            raise AssertionError('Unexpected recovery route: '+route)
        fixture.transport.get.side_effect=get;fixture.transport.archive.side_effect=lambda number:bodies[number]
        fixture.candidate_metadata,fixture.candidate_bodies,fixture.runs=metadata,bodies,runs
        verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':current_attempt},lambda sha:fixture.source,receipt_provider=fixture.receipt_fixture,candidate_outputs=fixture.candidate_outputs,proof_provider=fixture.proof_provider)
        return fixture,verifier,original,current

    def test_recovery_uses_existing_release_seal_verifier_with_original_producer(self):
        fixture,verifier,original,current=self.recovery_fixture()
        with mock.patch.object(module.entry.execution,'jobs_for',return_value=fixture.jobs), mock.patch.object(module.entry.execution,'fetch_json_artifact',return_value=(fixture.seal,fixture.artifact)):
            proof=verifier.verify(fixture.manifest,fixture.root)
        self.assertTrue(proof['prepublicationVerified'])
        self.assertEqual(fixture.manifest['producerAttempt'],2)
        original['status']='in_progress'
        with self.assertRaisesRegex(ValueError,'not terminal'):
            verifier.verify(fixture.manifest,fixture.root)

    def test_postpublish_joint_contract_binds_original_seal_new_acceptance_and_live_registry(self):
        fixture,verifier,original,current=self.recovery_fixture()
        registry=mock.Mock();registry.verify.return_value=fixture.public['packages']
        with mock.patch.object(module.entry.evidence.registry_module,'RegistryReadOnly',return_value=registry):
            proof=verifier.verify(fixture.manifest,fixture.root,phase='postpublish')
        self.assertTrue(proof['verified']);self.assertEqual(proof['producerAttempt'],2);self.assertEqual(proof['publicAttempt'],3)
        registry.verify.assert_called_once_with({row['kind']+':'+row['name']:row for row in fixture.manifest['packages']})

    def test_workflow_joint_contract_keeps_source_producer_attempt_and_explicit_writes(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        self.assertIn('group: farm-independent-release\n  cancel-in-progress: false',workflow)
        for stage in ['publisher','public-proof','finalize']:
            block=re.search(r'^  '+re.escape(stage)+r':\n(.*?)(?=^  [A-Za-z][A-Za-z0-9-]*:\n|\Z)',workflow,re.M|re.S).group(1)
            self.assertIn('needs.candidate-identity.outputs.attempt',block)
            self.assertRegex(block,r'needs:(?:[^\n]*candidate-identity|\n(?:      - [^\n]+\n)*      - candidate-identity)')
        identity=re.search(r'^  candidate-identity:\n(.*?)(?=^  [A-Za-z][A-Za-z0-9-]*:)',workflow,re.M|re.S).group(1)
        self.assertIn("needs.source.outputs.reuse == 'true' && needs.source.outputs.producer_attempt || needs.producer.outputs.candidate_attempt",identity)
        self.assertIn("needs.producer.result == 'success' || needs.source.outputs.reuse == 'true'",identity)
        self.assertIn("int(attempt) <= int(os.environ['GITHUB_RUN_ATTEMPT'])",identity)
        self.assertIn("inputs.publish == true",workflow)
        self.assertIn('name: '+module.ROLES['upload'],workflow)
        self.assertIn('name: '+module.ROLES['finalize'],workflow)
        native=(Path(__file__).parents[1]/'.github/workflows/release-native-build.yaml').read_text()
        self.assertIn('profile: release',native)
        self.assertNotIn('scripts/incremental-release-functional.py --source',workflow)
        self.assertNotIn('main-evidence:',workflow)
        retired=(Path(__file__).parents[1]/'.github/workflows/v2-stable-release.yaml').read_text(encoding='utf-8')
        self.assertNotIn('pull_request:',retired);self.assertNotIn('secrets.',retired);self.assertNotIn('v2-auto-release',retired)

    def test_existing_prerelease_latest_allows_stable_publication(self):
        # Real registry version observed on 2026-11-10; no registry writes.
        for version in ['1.0.0-nightly-20241024075304', '2.0.0-rc.1', '2.0.0-beta.1']:
            with self.subTest(version=version):
                missing = urllib.error.HTTPError('https://registry.npmjs.org/a/2.0.0', 404, 'missing', {}, None)
                writer, reader = self.registry([missing, {'name': 'a', 'version': version}], [])
                self.assertIsNone(writer.lookup({'kind': 'npm', 'name': 'a', 'version': '2.0.0'}))

    def test_comparison_cache_keys_exact_inputs_but_always_reads_registry(self):
        writer, reader = self.registry([{'name': 'a', 'version': v} for v in
            ['1.0.0-rc.1', '1.0.0-rc.1', '3.0.0-rc.1', '1.0.0-rc.1', 'bad', 'bad']], [])
        actual = module.subprocess.run
        with mock.patch.object(module.subprocess, 'run', wraps=actual) as calls:
            row = {'kind': 'npm', 'name': 'a', 'version': '2.0.0'}
            self.assertEqual(writer.latest_order(row), -1)
            self.assertEqual(writer.latest_order(row), -1)
            self.assertEqual(calls.call_count, 1)
            self.assertEqual(writer.latest_order(row), 1)
            self.assertEqual(writer.latest_order({**row, 'version': '1.0.0'}), -1)
            for _ in range(2):
                with self.assertRaisesRegex(ValueError, 'SemVer'): writer.latest_order(row)
            self.assertEqual(calls.call_count, 5)  # bad results are never cached
        self.assertEqual(reader.json.call_count, 6)

    def test_cached_missing_latest_still_fetches_and_detects_new_version(self):
        missing = urllib.error.HTTPError('https://registry.npmjs.org/a/latest', 404, 'absent', {}, None)
        writer, reader = self.registry([missing, missing, {'name': 'a', 'version': '3.0.0-rc.1'}], [])
        actual = module.subprocess.run
        with mock.patch.object(module.subprocess, 'run', wraps=actual) as calls:
            row = {'kind': 'npm', 'name': 'a', 'version': '2.0.0'}
            self.assertIsNone(writer.latest_order(row)); self.assertIsNone(writer.latest_order(row))
            self.assertEqual(calls.call_count, 1)
            self.assertEqual(writer.latest_order(row), 1)
            self.assertEqual(calls.call_count, 2)
        self.assertEqual(reader.json.call_count, 3)

    def test_npm_failure_diagnostics_are_bounded_and_do_not_repeat_writes(self):
        import traceback
        secret = 'fake-secret:/+credential'
        variants = [secret, urllib.parse.quote(secret, safe=''), base64.b64encode(secret.encode()).decode()]
        output = ('\n'.join(variants) + '\nAuthorization: Bearer hidden\n'
                  + '\x1b[31m::error::injected\x00' + 'x' * 100000 + '\nnpm error code E403\n').encode()
        row = {'kind': 'npm', 'name': '@farmfe/cli', 'version': '2.0.0', 'archive': 'a.tgz',
               'archiveSha256': hashlib.sha256(b'frozen').hexdigest()}
        for operation in ['publish', 'dist-tag', 'resolve-cli']:
            for timeout in [False, True]:
                with self.subTest(operation=operation, timeout=timeout):
                    writer, _ = self.registry([], [])
                    writer.environment['NPM_TOKEN'] = secret
                    failure = module.subprocess.TimeoutExpired(['secret-argv', secret], 120, output=output, stderr=output)
                    command = mock.Mock(side_effect=failure if timeout else None,
                                        return_value=SimpleNamespace(returncode=1, stdout=output, stderr=output))
                    writer.command = command
                    writer.latest_order = mock.Mock(return_value=-1)
                    if operation != 'resolve-cli': writer._npm = ('node', '/fixed/npm', {'HOME': '/tmp'})
                    with self.assertRaises(ValueError) as raised:
                        with writer:
                            if operation == 'publish': writer.publish(row, b'frozen')
                            elif operation == 'dist-tag': writer.confirm(row)
                            else: writer.npm()
                    diagnostic = ''.join(traceback.format_exception(raised.exception))
                    self.assertIn('operation=' + operation, diagnostic)
                    self.assertIn('timeout' if timeout else 'exit=1', diagnostic)
                    if operation != 'resolve-cli': self.assertIn('@farmfe/cli@2.0.0', diagnostic)
                    if not timeout: self.assertIn('E403', diagnostic)
                    for value in variants + ['secret-argv', 'hidden', 'injected', 'Authorization']:
                        self.assertNotIn(value, diagnostic)
                    self.assertLess(len(str(raised.exception)), 700)
                    self.assertEqual(command.call_count, 1)
                    self.assertTrue(writer._closed)

    def test_npm_diagnostic_parser_rejects_arbitrary_and_truncated_codes(self):
        row = {'name': 'a', 'version': '2.0.0'}
        for output in ['npm error code SECRET_VALUE', 'npm error code E403 secret',
                       'npm error code E403\x00',
                       'x' * 70000 + 'npm error code E403\n' + '\n' * (65536 - len('npm error code E403\n'))]:
            error = module.npm_failure('publish', row, code=1, stderr=output)
            self.assertIn('No recognized npm error code', str(error))
            self.assertNotIn('SECRET_VALUE', str(error))
        self.assertIn('E403:', str(module.npm_failure('publish', row, code=1,
                      stderr='x' * 100000 + '\nnpm error code E403\n')))
        error = module.npm_failure('publish', row, code=1,
                                  stderr='\x1b[31mnpm ERR! code EOTP\x1b[0m\r\n')
        self.assertIn('EOTP: Registry requires a one-time password', str(error))
        self.assertIn('invalid-identity', str(module.npm_failure(
            'publish', {'name': 'a\n::error::secret', 'version': '2.0.0'})))

    def test_npm_cli_exceptions_do_not_print_arguments_or_environment(self):
        import traceback
        for error in [module.subprocess.CalledProcessError(2, ['secret-cli', 'secret-token'],
                       output=b'secret-token', stderr=b'npm error code ENEEDAUTH\nsecret-token'),
                      OSError('secret-path secret-token')]:
            writer, _ = self.registry([], [])
            writer.environment['NPM_TOKEN'] = 'secret-token'
            writer.command = mock.Mock(side_effect=error)
            with self.assertRaises(ValueError) as raised:
                with writer: writer.npm()
            diagnostic = ''.join(traceback.format_exception(raised.exception))
            self.assertIn('operation=resolve-cli', diagnostic)
            self.assertNotIn('secret-', diagnostic)
            self.assertEqual(writer.command.call_count, 1)

    def test_invalid_diagnostic_identity_cannot_reveal_exception_context(self):
        import traceback
        for row in [{'name': 'Uppercase', 'version': '2.0.0'},
                    {'name': 'a' * 215, 'version': '2.0.0'},
                    {'name': 'a\nsecret-token', 'version': None}]:
            writer, _ = self.registry([], [])
            writer.command = mock.Mock(side_effect=module.subprocess.CalledProcessError(
                1, ['secret-token'], stderr=b'secret-token'))
            with self.assertRaises(ValueError) as raised:
                writer.npm_command('publish', row, ['unused'])
            diagnostic = ''.join(traceback.format_exception(raised.exception))
            self.assertIn('invalid-identity', diagnostic)
            self.assertNotIn('secret-token', diagnostic)
            self.assertEqual(writer.command.call_count, 1)

    def test_real_child_exit_and_timeout_are_sanitized(self):
        import sys
        import traceback
        row = {'name': 'a', 'version': '2.0.0'}
        for timeout in [False, True]:
            writer, _ = self.registry([], [])
            source = "import sys,time;sys.stderr.write('npm error code E403\\nsecret-fixture\\n');sys.stderr.flush();"
            source += 'time.sleep(5)' if timeout else 'sys.exit(7)'
            writer.command = mock.Mock(wraps=module.subprocess.run)
            with self.assertRaises(ValueError) as raised:
                writer.npm_command('publish', row, [sys.executable, '-c', source],
                                   capture_output=True, timeout=0.2 if timeout else 5)
            diagnostic = ''.join(traceback.format_exception(raised.exception))
            self.assertIn('timeout' if timeout else 'exit=7', diagnostic)
            self.assertIn('E403:', diagnostic)
            self.assertNotIn('secret-fixture', diagnostic)
            self.assertEqual(writer.command.call_count, 1)

    def test_manual_diagnostic_only_reads_fixed_target_and_hides_username(self):
        diagnostic = module.load('manual_npm_diagnostic_test', 'npm-release-diagnose.py')
        env = {'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'farm-fe/farm',
               'GITHUB_REF': 'refs/heads/main', 'GITHUB_EVENT_NAME': 'workflow_dispatch',
               'GITHUB_WORKFLOW_REF': 'farm-fe/farm/.github/workflows/npm-release-diagnose.yaml@refs/heads/main',
               'GITHUB_SHA': 'a' * 40, 'GITHUB_WORKFLOW_SHA': 'a' * 40, 'NPM_TOKEN': 'fake-secret'}
        writer = mock.MagicMock(); writer.__enter__.return_value = writer
        writer.npm.return_value = ('node', '/fixed/npm', {})
        writer.npm_command.return_value = SimpleNamespace(stdout=b'"private-username"')
        writer.reader.json.side_effect = urllib.error.HTTPError('fixed', 404, 'absent', {}, None)
        factory = mock.Mock(return_value=writer)
        report = diagnostic.diagnose(env, factory)
        self.assertTrue(report['authenticated']); self.assertFalse(report['targetExists'])
        self.assertFalse(report['publicationPermissionVerified']); self.assertFalse(report['publicationExecuted'])
        self.assertNotIn('private-username', json.dumps(report)); self.assertNotIn('fake-secret', json.dumps(report))
        self.assertEqual(writer.npm_command.call_args.args[0], 'whoami')
        self.assertEqual(writer.npm_command.call_args.args[2], ['node', '/fixed/npm', 'whoami', '--json', '--registry=https://registry.npmjs.org'])
        writer.reader.json.assert_called_once_with('https://registry.npmjs.org/%40farmfe%2Fcli/2.0.0')
        writer.publish.assert_not_called(); writer.confirm.assert_not_called()
        for key, value in [('GITHUB_REF', 'refs/heads/topic'), ('GITHUB_EVENT_NAME', 'push'),
                           ('GITHUB_REPOSITORY', 'other/farm'), ('GITHUB_WORKFLOW_SHA', 'b' * 40)]:
            factory.reset_mock()
            with self.assertRaises(ValueError): diagnostic.diagnose({**env, key: value}, factory)
            factory.assert_not_called()
        writer.npm_command.side_effect = diagnostic.publisher.npm_failure('whoami', diagnostic.PACKAGE, code=1, stderr='npm error code E401')
        report = diagnostic.diagnose(env, factory)
        self.assertFalse(report['authenticated']); self.assertIn('E401', report['authenticationError'])
        writer.npm_command.side_effect = ValueError('private-username fake-secret')
        report = diagnostic.diagnose(env, factory)
        self.assertEqual(report['authenticationError'], 'Authentication response invalid')

    def test_diagnostic_public_target_and_timeout_remain_bounded(self):
        diagnostic = module.load('manual_npm_diagnostic_edges', 'npm-release-diagnose.py')
        env = {'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'farm-fe/farm',
               'GITHUB_REF': 'refs/heads/main', 'GITHUB_EVENT_NAME': 'workflow_dispatch',
               'GITHUB_WORKFLOW_REF': 'farm-fe/farm/.github/workflows/npm-release-diagnose.yaml@refs/heads/main',
               'GITHUB_SHA': 'a' * 40, 'GITHUB_WORKFLOW_SHA': 'a' * 40, 'NPM_TOKEN': 'fake-secret'}
        writer = mock.MagicMock(); writer.__enter__.return_value = writer
        writer.npm.return_value = ('node', '/fixed/npm', {})
        writer.npm_command.side_effect = diagnostic.publisher.npm_failure(
            'whoami', diagnostic.PACKAGE, timed_out=True, stderr=b'Authorization: fake-secret')
        factory = mock.Mock(return_value=writer)
        for value in [{'name': '@farmfe/cli', 'version': '2.0.0'}, {'name': 'wrong', 'version': '2.0.0'},
                      urllib.error.HTTPError('fixed', 403, 'private-secret', {}, None)]:
            writer.reader.json.side_effect = value if isinstance(value, Exception) else None
            writer.reader.json.return_value = value
            report = diagnostic.diagnose(env, factory)
            self.assertFalse(report['authenticated']); self.assertIn('timeout', report['authenticationError'])
            self.assertFalse(report['publicationPermissionVerified'])
            self.assertNotIn('secret', json.dumps(report))
            if isinstance(value, dict) and value['name'] == '@farmfe/cli': self.assertTrue(report['targetExists'])
            else: self.assertIsNone(report['targetExists']); self.assertIn('registryError', report)
        factory.reset_mock()
        with self.assertRaises(ValueError): diagnostic.diagnose({**env, 'GITHUB_WORKFLOW_REF': 'wrong'}, factory)
        factory.assert_not_called()

    def test_diagnostic_workflow_is_manual_read_only_and_has_no_package_input(self):
        workflow = (Path(__file__).parents[1] / '.github/workflows/npm-release-diagnose.yaml').read_text()
        self.assertIn('workflow_dispatch:', workflow)
        self.assertIn("github.ref == 'refs/heads/main'", workflow)
        self.assertIn('ref: ${{ github.sha }}', workflow)
        self.assertIn('contents: read', workflow)
        for forbidden in ['write', 'inputs:', 'publish', 'dist-tag', 'pull_request:', 'push:', 'npm install', 'pnpm install']:
            self.assertNotIn(forbidden, workflow.replace('without publishing', ''))

    def test_failed_config_preparation_cleans_owned_directory(self):
        writer, _ = self.registry([], [])
        writer.environment['NPM_TOKEN'] = 'fixture-only'
        writer.command = mock.Mock(return_value=SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0))
        actual = tempfile.TemporaryDirectory; paths = []
        def temporary(**options):
            directory = actual(**options); paths.append(Path(directory.name)); return directory
        with mock.patch.object(module.tempfile, 'TemporaryDirectory', side_effect=temporary):
            with mock.patch.object(Path, 'write_text', side_effect=OSError('fixture config failure')):
                with self.assertRaisesRegex(OSError, 'config failure'):
                    with writer: writer.npm()
        self.assertEqual(len(paths), 1)
        self.assertFalse(paths[0].exists())

    def test_npm_preparation_is_reused_and_cleaned_on_success_and_failure(self):
        for fail in [False, True]:
            writer, _ = self.registry([], [])
            writer.environment.update(NPM_TOKEN='fixture-secret', GH_TOKEN='github-fixture')
            commands = []; paths = []
            def command(argv, **options):
                commands.append(argv)
                if '--input-type=module' in argv:
                    return SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0)
                config = Path(options['env']['NPM_CONFIG_USERCONFIG']); paths.append(config)
                self.assertNotIn('fixture-secret', config.read_text())
                self.assertNotIn('_auth', config.read_text())
                self.assertEqual(Path(options['env']['NPM_CONFIG_GLOBALCONFIG']).read_text(), '')
                self.assertNotIn('GH_TOKEN', options['env'])
                self.assertNotIn('NPM_TOKEN', options['env'])
                self.assertNotIn('NODE_AUTH_TOKEN', options['env'])
                self.assertEqual(options['env']['ACTIONS_ID_TOKEN_REQUEST_TOKEN'], 'oidc-fixture')
                self.assertEqual(options['cwd'], str(config.parent))
                return SimpleNamespace(returncode=1 if fail else 0)
            writer.command = command
            body = b'frozen'
            row = {'kind': 'npm', 'name': 'a', 'version': '2.0.0', 'archive': 'a.tgz',
                   'archiveSha256': hashlib.sha256(body).hexdigest()}
            try:
                with writer:
                    writer.publish(row, body)
                    writer.publish(row, body)
            except ValueError:
                if not fail: raise
            self.assertEqual(sum('--input-type=module' in argv for argv in commands), 1)
            self.assertEqual(len(set(paths)), 1)
            self.assertTrue(all(not path.parent.exists() for path in paths))
            writer.close()

    def test_oidc_credentials_are_detached_from_parent_environment(self):
        environment = {'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://oidc.invalid/',
                       'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'secret', 'GH_TOKEN': 'github'}
        saved = module.detach_oidc_credentials(environment)
        self.assertEqual(set(saved), {'ACTIONS_ID_TOKEN_REQUEST_URL', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN'})
        self.assertEqual(environment, {'GH_TOKEN': 'github'})
        self.assertEqual(module.detach_oidc_credentials(environment), {})

    def test_oidc_is_required_and_ambient_credentials_never_reach_npm(self):
        writer, _ = self.registry([], [])
        writer.environment.pop('ACTIONS_ID_TOKEN_REQUEST_TOKEN')
        writer.command = mock.Mock()
        with self.assertRaisesRegex(ValueError, 'OIDC credentials missing'): writer.npm()
        writer.command.assert_not_called()
        writer, _ = self.registry([], [])
        writer.environment.update(NPM_TOKEN='legacy', NODE_AUTH_TOKEN='legacy', NPM_ID_TOKEN='legacy',
                                  npm_config_registry='https://attacker.invalid', NODE_OPTIONS='--require evil',
                                  GH_TOKEN='github-secret', GITHUB_TOKEN='github-secret', HOME='/unsafe',
                                  GITHUB_SHA='a' * 40)
        calls = []
        def command(argv, **options):
            calls.append((argv, options))
            return SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0)
        writer.command = command
        _, _, env = writer.npm()
        for key in ['NPM_TOKEN', 'NODE_AUTH_TOKEN', 'NPM_ID_TOKEN', 'npm_config_registry',
                    'NODE_OPTIONS', 'GH_TOKEN', 'GITHUB_TOKEN']:
            self.assertNotIn(key, env)
        self.assertNotIn('ACTIONS_ID_TOKEN_REQUEST_TOKEN', calls[0][1]['env'])
        self.assertEqual(env['ACTIONS_ID_TOKEN_REQUEST_TOKEN'], 'oidc-fixture')
        self.assertEqual(env['GITHUB_SHA'], 'a' * 40)
        self.assertNotEqual(env['HOME'], '/unsafe')
        self.assertIn('resolvePublisherNpmCli', calls[0][0][-1])
        writer.close()

    def test_all_archive_metadata_is_checked_before_publication(self):
        fixtures = module.entry.load('npm_tarball_test_fixtures', 'release-npm-tarball.test.py')
        good = fixtures.archive()
        bad = fixtures.archive(fixtures.manifest(repository='https://example.invalid/repo'))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = []
            for index, body in enumerate([good, bad]):
                name = str(index) + '.tgz'
                (root / name).write_bytes(body)
                rows.append({'kind': 'npm', 'name': '@farmfe/core', 'version': '2.0.0',
                             'archive': name, 'archiveBytes': len(body),
                             'archiveSha256': hashlib.sha256(body).hexdigest()})
            with self.assertRaisesRegex(ValueError, 'provenance repository'):
                module.validate_npm_archives({'packages': rows}, root)
            (root / '1.tgz').write_bytes(good)
            rows[1].update(archiveBytes=len(good), archiveSha256=hashlib.sha256(good).hexdigest())
            module.validate_npm_archives({'packages': rows}, root)

    def test_publish_uses_validated_snapshot_even_if_candidate_changes(self):
        writer, _ = self.registry([], [])
        body = b'validated frozen bytes'
        row = {'kind': 'npm', 'name': 'a', 'version': '2.0.0', 'archive': 'a.tgz',
               'archiveSha256': hashlib.sha256(body).hexdigest()}
        with tempfile.TemporaryDirectory() as directory:
            writer.directory = Path(directory)
            original = writer.directory / row['archive']
            original.write_bytes(body)
            seen = []
            def command(argv, **options):
                if '--input-type=module' in argv:
                    original.write_bytes(b'replaced after verification')
                    return SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0)
                target = Path(argv[argv.index('publish') + 1])
                seen.append((target.read_bytes(), target, options['cwd']))
                self.assertNotEqual(target, original)
                return SimpleNamespace(returncode=0)
            writer.command = command
            writer.publish(row, body)
            self.assertEqual(seen[0][0], body)
            self.assertFalse(seen[0][1].exists())

    def test_invalid_archive_is_rejected_before_oidc_or_write(self):
        writer, _ = self.registry([], [])
        # Restore the real validator; model fixtures otherwise use synthetic bytes.
        writer.validate_archive = lambda row, body: module.npm_tarball.validate(row, body)
        writer.command = mock.Mock()
        body = b'not a tarball'
        row = {'kind': 'npm', 'name': 'a', 'version': '2.0.0', 'archive': 'a.tgz',
               'archiveSha256': hashlib.sha256(body).hexdigest()}
        with self.assertRaises(ValueError): writer.publish(row, body)
        writer.command.assert_not_called()

    def test_prerelease_confirmation_promotes_without_republishing(self):
        writer, reader = self.registry([{'name': 'a', 'version': '2.0.0-rc.1'},
                                       {'name': 'a', 'version': '2.0.0-rc.1'},
                                       {'name': 'a', 'version': '2.0.0'}], [])
        writer.environment['NPM_TOKEN'] = 'fixture-only'
        writer.command = mock.Mock(side_effect=[mock.Mock(stdout=str(Path(__file__).resolve()), returncode=0), mock.Mock(returncode=0)])
        writer.confirm({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})
        self.assertEqual(writer.command.call_count, 2)
        self.assertIn('dist-tag', writer.command.call_args.args[0])
        self.assertEqual(reader.json.call_count, 3)

    def test_newer_prerelease_blocks_preflight_and_never_changes_tag(self):
        for version in ['2.1.0-rc.1', '3.0.0-rc.1', '3.0.0']:
            with self.subTest(version=version):
                missing = urllib.error.HTTPError('https://registry.npmjs.org/a/2.0.0', 404, 'missing', {}, None)
                writer, reader = self.registry([missing, {'name': 'a', 'version': version}], [])
                with self.assertRaisesRegex(ValueError, 'backwards'):
                    writer.lookup({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})
                writer, reader = self.registry([{'name': 'a', 'version': version}], [])
                writer.command = mock.Mock()
                writer.confirm({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})
                writer.command.assert_not_called()

    def test_tag_preparation_rereads_latest_and_preserves_a_concurrent_higher_version(self):
        writer, reader = self.registry([{'name': 'a', 'version': '2.0.0-rc.1'},
                                       {'name': 'a', 'version': '2.1.0-rc.1'}], [])
        writer.environment['NPM_TOKEN'] = 'fixture-only'
        writer.command = mock.Mock(return_value=mock.Mock(stdout=str(Path(__file__).resolve()), returncode=0))
        writer.confirm({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})
        self.assertEqual(writer.command.call_count, 1)  # resolve CLI only; no dist-tag mutation
        self.assertEqual(reader.json.call_count, 2)

    def test_latest_readback_rejects_a_lower_or_invalid_tag_after_command(self):
        for version in ['1.0.0', '2.0.0-rc.1', 'not-semver']:
            with self.subTest(version=version):
                writer, reader = self.registry([{'name': 'a', 'version': '1.0.0'},
                                               {'name': 'a', 'version': '1.0.0'},
                                               {'name': 'a', 'version': version}], [])
                writer.environment['NPM_TOKEN'] = 'fixture-only'
                writer.command = mock.Mock(side_effect=[mock.Mock(stdout=str(Path(__file__).resolve()), returncode=0), mock.Mock(returncode=0)])
                with self.assertRaises(ValueError):
                    writer.confirm({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})

    def test_missing_latest_and_invalid_identity_or_versions(self):
        missing = urllib.error.HTTPError('https://registry.npmjs.org/a/latest', 404, 'missing', {}, None)
        writer, reader = self.registry([missing, missing], [])
        self.assertIsNone(writer.lookup({'kind': 'npm', 'name': 'a', 'version': '2.0.0'}))
        for metadata in [{'name': 'other', 'version': '1.0.0'}, {'name': 'a'},
                         {'name': 'a', 'version': '01.0.0'}, {'name': 'a', 'version': '2.0.0-01'}]:
            writer, reader = self.registry([missing, metadata], [])
            with self.assertRaises(ValueError):
                writer.lookup({'kind': 'npm', 'name': 'a', 'version': '2.0.0'})
        for target in ['2.0.0-rc.1', '2.0.0+build', '02.0.0']:
            writer, reader = self.registry([missing, missing], [])
            with self.assertRaises(ValueError):
                writer.lookup({'kind': 'npm', 'name': 'a', 'version': target})

    def test_latest_confirmation_preserves_higher_version_without_command(self):
        writer,reader=self.registry([{'name':'a','version':'3.0.0'}],[])
        writer.command=mock.Mock()
        writer.confirm({'kind':'npm','name':'a','version':'2.0.0'})
        writer.command.assert_not_called()

    def public_fixture(self):
        manifest={'chain':'release','profile':'release','sourceSHA':'a'*40,'sourceTree':'b'*40,'producerRunId':7,'producerAttempt':1,'planHash':'c'*64}
        runs={attempt:{'id':7,'run_attempt':attempt,'path':module.entry.WORKFLOW,'head_sha':manifest['sourceSHA'],
              'head_branch':'main','event':'push','repository':{'id':507542208},'head_repository':{'id':507542208}}
              for attempt in [1,2]}
        jobs={1:[],2:[]};artifacts={};bodies={};reports={}
        for index,(platform,attempt) in enumerate([('ubuntu-24.04',1),('macos-latest',1),('windows-latest',2)],1):
            job={'id':index+10,'name':'Independent public registry consumer ('+platform+')','run_id':7,'run_attempt':attempt,
                 'head_sha':manifest['sourceSHA'],'status':'completed','conclusion':'success','steps':[{'name':module.PUBLIC_UPLOAD,'status':'completed','conclusion':'success',
                 'started_at':'1','completed_at':'3'}]}
            jobs[attempt].append(job)
            report={'status':'passed','publicRegistry':True,'workspacePatches':False,'cleanupConfirmed':True,
                    'stages':['npm-install','esm-cjs','types','docs-build','sdk'],
                    'publicationBinding':module.report_binding(manifest,7,attempt,platform)}
            reports[index]=report
            output=io.BytesIO()
            with zipfile.ZipFile(output,'w') as file:file.writestr('result.json',json.dumps(report))
            bodies[index]=output.getvalue()
            artifacts[index]={'id':index,'name':'release-public-'+platform+'-'+str(attempt),'expired':False,'created_at':'2',
                              'workflow_run':{'id':7},'size_in_bytes':len(bodies[index]),'digest':'sha256:'+hashlib.sha256(bodies[index]).hexdigest()}
        jobs[2].extend([{**job,'id':job['id']+100,'run_attempt':2} for job in jobs[1]])
        def get(route):
            if route=='':return {'id':5,'default_branch':'main'}
            if route=='/actions/runs/7/artifacts?per_page=100&page=1':return {'artifacts':list(artifacts.values())}
            if '/jobs?' in route:return {'jobs':jobs[int(route.split('/attempts/')[1].split('/')[0])]}
            if '/attempts/' in route:return runs[int(route.rsplit('/',1)[1])]
            if route.startswith('/actions/artifacts/'):return artifacts[int(route.rsplit('/',1)[1])]
            raise AssertionError('Unexpected public report API route: '+route)
        transport=mock.Mock();transport.get.side_effect=get;transport.archive.side_effect=lambda number:bodies[number]
        return manifest,transport,jobs,artifacts,bodies,reports,runs

    def public_outputs(self,artifacts):
        return {platform:{'artifact_id':str(index),'artifact_digest':artifacts[index]['digest'][7:],
                          'run_attempt':'2' if index==3 else '1'}
                for index,platform in enumerate(module.PUBLIC_PLATFORMS,1)}

    def test_mixed_attempt_public_matrix_is_bound_independently_for_each_os(self):
        manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture()
        output=Path(tempfile.mkdtemp(prefix='farm-public-mixed-contract-'))/'reports'
        with mock.patch.object(module.entry.execution,'jobs_for',side_effect=lambda client,run:jobs[run['run_attempt']]):
            selection=module.collect_public_reports(transport,manifest,{'runId':7,'attempt':2},output,upload_outputs=self.public_outputs(artifacts))
            verified=module.verify_public_selection(transport,manifest,7,2,selection['reports'],upload_outputs=self.public_outputs(artifacts))
        self.assertEqual({row['platform']:row['attempt'] for row in verified},{'ubuntu-24.04':1,'macos-latest':1,'windows-latest':2})
        for platform in module.PUBLIC_PLATFORMS:self.assertTrue((output/('release-public-'+platform)/'result.json').is_file())

    def test_duplicate_os_selection_and_forged_report_os_are_rejected(self):
        manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture()
        with self.assertRaisesRegex(ValueError,'one independent report'):
            module.verify_public_selection(transport,manifest,7,2,[{'platform':'ubuntu-24.04'}]*3)
        reports[3]['publicationBinding']['platform']='ubuntu-24.04'
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as file:file.writestr('result.json',json.dumps(reports[3]))
        bodies[3]=output.getvalue();artifacts[3].update({'size_in_bytes':len(bodies[3]),'digest':'sha256:'+hashlib.sha256(bodies[3]).hexdigest()})
        with mock.patch.object(module.entry.execution,'jobs_for',side_effect=lambda client,run:jobs[run['run_attempt']]):
            with self.assertRaisesRegex(ValueError,'Public receipt frozen bytes'):
                module.verify_public_report(transport,manifest,7,2,'windows-latest',3,trusted_output=self.public_outputs(artifacts)['windows-latest'])

    def test_wrong_public_source_or_artifact_digest_cannot_resume(self):
        for field,value in [('head_sha','f'*40),('event','pull_request'),('head_repository',{'id':99})]:
            manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture();runs[2][field]=value
            with self.assertRaisesRegex(ValueError,'run or source differs'):
                module.verify_public_report(transport,manifest,7,2,'windows-latest',3,trusted_output=self.public_outputs(artifacts)['windows-latest'])
        manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture();artifacts[3]['digest']='sha256:'+'f'*64
        with mock.patch.object(module.entry.execution,'jobs_for',side_effect=lambda client,run:jobs[run['run_attempt']]):
            with self.assertRaisesRegex(ValueError,'digest differs'):module.verify_public_report(transport,manifest,7,2,'windows-latest',3,trusted_output=self.public_outputs(artifacts)['windows-latest'])

    def frozen_fixture(self):
        fixture,verifier,original,current=self.recovery_fixture()
        spec=importlib.util.spec_from_file_location('state_frozen_fixture',Path(__file__).with_name('incremental-publication-state.test.py'))
        shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
        store=module.state.GitRefStore(shared.GitAPI())
        owner={'runId':10,'attempt':2,'repositoryId':fixture.repository['id'],'sourceSHA':fixture.source['sha'],'sourceTree':fixture.source['tree'],
               'workflow':module.entry.WORKFLOW,'policyHash':fixture.source['policyHash'],'producerRunId':10,'producerAttempt':2}
        first=module.state.Publication(store,fixture.manifest,owner,lambda value:True);first.acquire()
        return fixture,verifier,original,current,shared,store,owner,first

    def test_failed_a_then_new_main_policy_b_recovers_frozen_a_and_unblocks_b(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        registry=shared.Registry();registry.crash_after=fixture.manifest['packages'][0]['name']
        read=lambda row:(fixture.root/row['archive']).read_bytes()
        with self.assertRaises(RuntimeError):first.upload(read,registry)
        newer={**copy.deepcopy(fixture.source),'sha':'d'*40,'tree':'e'*40,'policyHash':'new-main-policy'}
        old_get=fixture.transport.get.side_effect
        fixture.transport.get.side_effect=lambda route: {'commit':{'sha':newer['sha']}} if route=='/branches/main' else old_get(route)
        frozen=module.frozen_authorization(store,fixture.manifest,fixture.source,fixture.repository['id'],{'runId':10,'attempt':3})
        verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':3},lambda sha:newer if sha==newer['sha'] else fixture.source,frozen=frozen,receipt_provider=fixture.receipt_fixture,candidate_outputs=fixture.candidate_outputs,proof_provider=fixture.proof_provider)
        b=copy.deepcopy(fixture.manifest);b.update({'sourceSHA':newer['sha'],'sourceTree':newer['tree'],'planHash':'f'*64,'producerRunId':11,'producerAttempt':1})
        b_owner={**owner,'runId':11,'attempt':1,'sourceSHA':newer['sha'],'sourceTree':newer['tree'],'policyHash':newer['policyHash'],'producerRunId':11,'producerAttempt':1}
        with self.assertRaisesRegex(ValueError,'original manifest'):module.state.Publication(store,b,b_owner,lambda value:True).acquire()
        with mock.patch.object(module.entry.execution,'jobs_for',return_value=fixture.jobs),mock.patch.object(module.entry.execution,'fetch_json_artifact',return_value=(fixture.seal,fixture.artifact)):
            self.assertTrue(verifier.verify(fixture.manifest,fixture.root)['prepublicationVerified'])
        registry.crash_after=None
        retry=module.state.Publication(store,fixture.manifest,{**owner,'attempt':3},lambda value:True)
        retry.recover(lambda previous:module.terminal_owner(fixture.transport,{'runId':10,'attempt':3},fixture.source,previous))
        retry.upload(read,registry)
        proof=lambda value:{'manifestHash':module.state.digest(value),'verified':True}
        retry.finalize(proof,lambda value,proof:{'manifestHash':module.state.digest(value),'sourceSHA':value['sourceSHA'],'verified':True})
        self.assertEqual(store.read()[1]['baseline'],module.state.digest(fixture.manifest))
        self.assertEqual(store.read()[1]['completed'][module.state.digest(fixture.manifest)]['authorization'],owner)
        b['baseline']=module.state.digest(fixture.manifest)
        following=module.state.Publication(store,b,b_owner,lambda value:True);following.acquire()
        self.assertEqual(store.read()[1]['active']['transaction'],module.state.digest(b))

    def test_forged_frozen_policy_or_original_pr_origin_cannot_bypass_trust(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        revision,saved=store.read();saved['active']['authorization']['policyHash']='forged-old-policy';store.swap(revision,saved)
        with self.assertRaisesRegex(ValueError,'authorization or source policy differs'):
            module.frozen_authorization(store,fixture.manifest,fixture.source,fixture.repository['id'],{'runId':10,'attempt':3})
        for field,value in [('event','pull_request'),('path','.github/workflows/ci.yaml'),('head_repository',{'id':99})]:
            fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
            frozen=module.frozen_authorization(store,fixture.manifest,fixture.source,fixture.repository['id'],{'runId':10,'attempt':3})
            original[field]=value
            verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':3},lambda sha:fixture.source,frozen=frozen,receipt_provider=fixture.receipt_fixture,candidate_outputs=fixture.candidate_outputs,proof_provider=fixture.proof_provider)
            with self.assertRaisesRegex(ValueError,'not terminal or its source differs'):verifier.verify(fixture.manifest,fixture.root)

    def test_registry_readback_uses_original_frozen_attempt_without_functional_consumers(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        self.assertNotIn('consumer_attempt:',workflow);self.assertNotIn('outputs.consumer_attempt',workflow)
        self.assertNotIn('  public-consumers:',workflow)
        block=workflow.split('  public-proof:',1)[1].split('  sign-public-acceptance:',1)[0]
        self.assertIn('needs.candidate-identity.outputs.attempt',block)
        self.assertIn('incremental-release-record.py registry --candidate release-candidate',block)
        self.assertNotIn('pnpm',block);self.assertNotIn('node ',block)
        self.assertNotIn('collect-public --candidate release-candidate',workflow)
        self.assertNotIn('bind-public-proof --candidate release-candidate',workflow)

    def test_plan_is_recomputed_after_acquisition_under_the_owned_fence(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        calls=[]
        def compute(source,baseline):
            first.owned();calls.append((source['sha'],baseline))
            return module.entry.source_plan(source,baseline)
        result=module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=compute)
        self.assertEqual(result['planHash'],fixture.manifest['planHash']);self.assertEqual(calls,[(fixture.source['sha'],None)])

    def test_locked_plan_mismatch_releases_only_an_unwritten_candidate(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        fresh=module.entry.source_plan(fixture.source);fresh['planHash']='f'*64
        with self.assertRaisesRegex(ValueError,'prepare a new candidate'):
            module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=lambda source,baseline:fresh)
        saved=store.read()[1];self.assertIsNone(saved['active']);self.assertIsNone(saved['baseline'])
        self.assertIn(module.state.digest(fixture.manifest),saved['rejected'])
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        first.record('cargo:started','upload-started')
        with self.assertRaisesRegex(ValueError,'preserve uploaded originals'):
            module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=lambda source,baseline:fresh)
        self.assertIsNotNone(store.read()[1]['active']);first.owned()

    def test_empty_publish_is_an_explicit_nonpublishing_nonbaseline_path(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        noop=workflow.split('  no-publication:',1)[1].split('  verify:',1)[0]
        self.assertIn("publish_required == 'false'",noop)
        self.assertIn("plan['publish'] == []",noop);self.assertIn("'baselineAdvanced':False",noop)
        self.assertNotIn('secrets.',noop);self.assertNotIn('contents: write',noop)
        producer=workflow.split('  producer:',1)[1].split('  candidate-consumers:',1)[0]
        self.assertIn("needs.plan.outputs.build_required == 'true'",producer)
        self.assertIn("plan['build'] == []",noop)
        self.assertIn("verify-rebuild release-candidate",producer)
        self.assertIn("farm-release-rebuild-check-",producer)

    def test_candidate_success_in_intermediate_attempt_survives_a_third_attempt(self):
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        proof=verifier.verify(fixture.manifest,fixture.root)
        self.assertTrue(proof['prepublicationVerified'])
        self.assertEqual({platform:row['attempt'] for platform,row in proof['candidateReports'].items()},
                         {'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        baseline=module.entry.load('joint_candidate_baseline','incremental-release-baseline.py').ReleaseBaselineReader(fixture.transport,None,lambda sha:fixture.source)
        self.assertEqual(proof['candidateReports'],baseline.candidates(fixture.manifest,fixture.repository,3,trusted_outputs=fixture.candidate_outputs))

    def test_candidate_middle_attempt_rejects_wrong_source_and_wrong_host_report(self):
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        fixture.runs[2]['head_sha']='f'*40
        with self.assertRaisesRegex(ValueError,'Explicit producer run identity'):verifier.verify(fixture.manifest,fixture.root)
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        report=json.loads(zipfile.ZipFile(io.BytesIO(fixture.candidate_bodies[200])).read('result.json'))
        report['platform']={'os':'win32','arch':'x64','abi':'win32-x64-msvc'}
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as file:file.writestr('result.json',json.dumps(report))
        body=output.getvalue();fixture.candidate_bodies[200]=body
        fixture.candidate_metadata[200].update({'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()})
        with self.assertRaisesRegex(ValueError,'Artifact differs from fixed upload action outputs'):verifier.verify(fixture.manifest,fixture.root)

    def test_locked_planner_exception_releases_only_durably_unwritten_claim(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        def unavailable(source,baseline):raise OSError('read-only plan inputs unavailable')
        with self.assertRaisesRegex(OSError,'inputs unavailable'):
            module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=unavailable)
        saved=store.read()[1];self.assertIsNone(saved['active'])
        self.assertEqual(saved['rejected'][module.state.digest(fixture.manifest)]['reason'],'locked-plan-unavailable-before-registry-write')
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        first.record('cargo:uncertain','upload-started')
        with self.assertRaises(OSError):module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=unavailable)
        self.assertIsNotNone(store.read()[1]['active']);first.owned()

    def test_locked_baseline_read_failure_and_unprovable_state_never_guess_upload_status(self):
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        api=shared.GitAPI();store=module.state.GitRefStore(api);baseline='e'*64
        store.swap(None,{'baseline':baseline,'active':None,'completed':{}})
        manifest={**fixture.manifest,'baseline':baseline}
        first=module.state.Publication(store,manifest,owner,lambda value:True);first.acquire()
        def denied(client,transaction):raise PermissionError('baseline read unavailable')
        with self.assertRaises(PermissionError):module.verify_locked_plan(first,fixture.source,mock.Mock(),baseline_reader=denied)
        self.assertIsNone(store.read()[1]['active'])
        fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
        read=store.read;count=0
        def unknown():
            nonlocal count
            count+=1
            if count>1:raise OSError('cannot prove durable upload status')
            return read()
        with mock.patch.object(store,'read',side_effect=unknown):
            with self.assertRaisesRegex(OSError,'durable upload status'):
                module.verify_locked_plan(first,fixture.source,mock.Mock(),planner=lambda source,baseline: (_ for _ in ()).throw(ValueError('plan unavailable')))
        self.assertIsNotNone(read()[1]['active'])

    def test_candidate_tests_run_on_pr_while_main_retains_frozen_seal_and_registry_proof(self):
        root=Path(__file__).parents[1]
        workflow=(root/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        self.assertNotIn('  candidate-consumers:',workflow)
        self.assertIn('name: Upload independent release seal',workflow)
        self.assertIn('name: Freeze independent release candidates',workflow)
        self.assertIn('name: Upload independent release public acceptance',workflow)
        ci=(root/'.github/workflows/ci.yaml').read_text()
        self.assertIn('uses: ./.github/workflows/v2-candidate-seal.yaml',ci)
        candidate=(root/'.github/workflows/v2-candidate-seal.yaml').read_text()
        self.assertIn('verify-v2-npm-consumer.mjs',candidate)
        self.assertIn('run-sdk-consumer.mjs',candidate)

    def test_locked_plan_hash_ignores_only_state_revision_not_baseline_content(self):
        shared=module.entry.load('joint_plan_baseline_fixture','incremental-release-baseline.test.py')
        fixture=shared.Contracts();fixture.setUp();baseline=fixture.reader.lookup()
        first=module.entry.source_plan(fixture.source,baseline)
        advanced=copy.deepcopy(baseline);advanced['stateRevision']='f'*40
        second=module.entry.source_plan(fixture.source,advanced)
        self.assertEqual(first['planHash'],second['planHash'])
        changed=copy.deepcopy(advanced)
        key=next(iter(changed['ledger']['packages']));changed['ledger']['packages'][key]['archiveSha256']='d'*64
        with self.assertRaises(ValueError):module.entry.source_plan(fixture.source,changed)

    def test_unlocked_resume_uses_the_same_main_history_api_as_current_entry(self):
        fixture,verifier,original,current=self.recovery_fixture()
        newer={**fixture.source,'sha':'d'*40,'policyHash':'new-main-policy'}
        get=fixture.transport.get.side_effect
        comparison={'status':'ahead','base_commit':{'sha':fixture.source['sha']},'merge_base_commit':{'sha':fixture.source['sha']}}
        def history(route):
            if route=='/branches/main':return {'commit':{'sha':newer['sha']}}
            if route=='/compare/'+fixture.source['sha']+'...'+newer['sha']:return comparison
            return get(route)
        http_fixture=module.entry.load('restore_http_fixture','ci-native-evidence.test.py')
        with http_fixture.local_api(history) as (client, seen):
            # Keep the existing signed archive fixture boundary; all JSON reads,
            # including main ancestry, use the production HTTP adapter.
            client.archive=fixture.transport.archive
            verifier=module.RecoveryVerifier(client,{'runId':10,'attempt':3},lambda sha:newer if sha==newer['sha'] else fixture.source,receipt_provider=fixture.receipt_fixture,candidate_outputs=fixture.candidate_outputs,proof_provider=fixture.proof_provider)
            self.assertTrue(verifier.verify(fixture.manifest,fixture.root)['prepublicationVerified'])
            self.assertIn('/compare/'+fixture.source['sha']+'...'+newer['sha'], seen)
            comparison['status']='diverged'
            with self.assertRaisesRegex(ValueError,'trusted main history'):verifier.verify(fixture.manifest,fixture.root)


class LatestHTTPContracts(unittest.TestCase):
    """Real read-only HTTP/parser + publisher/state algorithms; writes stay in memory."""
    def fixture(self):
        shared = module.load('latest_http_state_fixture', 'incremental-publication-state.test.py')
        api, store, manifest, _, _ = shared.fixture()
        names = ['@farmfe/plugin-replace-dirname-darwin-arm64', '@farmfe/plugin-dts-darwin-arm64']
        bodies = {name: ('frozen fixture archive ' + name).encode() for name in names}
        manifest['packages'] = [{'kind': 'npm', 'name': name, 'version': version, 'archive': str(i) + '.tgz',
            'archiveSha256': hashlib.sha256(bodies[name]).hexdigest(), 'archiveBytes': len(bodies[name])}
            for i, (name, version) in enumerate(zip(names, ['2.0.0', '1.0.0']))]
        # Versions from the 2026-11-10 read-only registry snapshot. Bodies are fixtures.
        registry = SimpleNamespace(values={}, latest=dict(zip(names, ['1.0.0-nightly-20241024075304',
            '1.0.0-nightly-20250410022118'])), writes=[], tags=[], requests=[], hook=None, lose=None)
        rows = {row['name']: row for row in manifest['packages']}
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args): pass
            def do_GET(self):
                parts = self.path.lstrip('/').split('/')
                name = urllib.parse.unquote(parts[0]); selector = parts[1]
                registry.requests.append((name, selector))
                if registry.hook: registry.hook(name, selector)
                body = None
                if selector == 'latest' and name in registry.latest:
                    body = json.dumps({'name': name, 'version': registry.latest[name]}).encode()
                elif selector == '-' and name in registry.values:
                    body = registry.values[name]
                elif name in registry.values:
                    body = json.dumps({'name': name, 'version': rows[name]['version'], 'dist': {
                        'tarball': 'https://registry.npmjs.org/' + urllib.parse.quote(name, safe='') + '/-/archive.tgz',
                        'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(registry.values[name]).digest()).decode()}}).encode()
                self.send_response(200 if body is not None else 404); self.end_headers()
                self.wfile.write(body or b'{}')
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        def close(): server.shutdown(); server.server_close(); thread.join()
        self.addCleanup(close)
        class Relay:
            def open(self, request, timeout):
                parsed = urllib.parse.urlsplit(request.full_url)
                assert parsed.hostname == 'registry.npmjs.org' and request.get_method() == 'GET'
                return urllib.request.urlopen('http://127.0.0.1:' + str(server.server_port) + parsed.path, timeout=timeout)
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        for row in manifest['packages']: (root / row['archive']).write_bytes(bodies[row['name']])
        def command(argv, **options):
            if '--input-type=module' in argv:
                return SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0)
            if 'publish' in argv:
                filename = Path(argv[argv.index('publish') + 1])
                row = next(row for row in rows.values() if row['archive'] == filename.name)
                self.assertIn('farm-release-' + row['version'], argv)
                registry.values[row['name']] = filename.read_bytes(); registry.writes.append(row['name'])
                if registry.lose == row['name']:
                    registry.lose = None
                    raise OSError('fixture registry accepted bytes but response lost')
            else:
                self.assertIn('dist-tag', argv)
                package = argv[argv.index('add') + 1]; name, version = package.rsplit('@', 1)
                registry.latest[name] = version; registry.tags.append(name)
            return SimpleNamespace(returncode=0)
        writer = module.RegistryWriter({'GITHUB_ACTIONS': 'true', 'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://oidc.invalid/', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'oidc-fixture', 'NPM_TOKEN': 'fixture-only'}, root, command)
        writer.validate_archive = mock.Mock()  # synthetic bytes in registry-model tests
        self.addCleanup(writer.close)
        writer.reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'}, opener=Relay())
        def runner(attempt):
            return module.state.Publication(store, manifest, {'runId': 7, 'attempt': attempt}, lambda value: True)
        return store, manifest, bodies, registry, writer, runner

    def test_partial_upload_lost_response_resumes_exact_bytes_then_skips_existing(self):
        store, manifest, bodies, registry, writer, runner = self.fixture()
        first_name = manifest['packages'][0]['name']; registry.lose = first_name
        first = runner(1); first.acquire()
        with self.assertRaisesRegex(ValueError, 'outcome uncertain'):
            first.upload(lambda row: bodies[row['name']], writer)
        self.assertEqual(registry.writes, [first_name])
        self.assertEqual(store.read()[1]['active']['packages']['npm:' + first_name], 'upload-started')
        second = runner(2); second.recover(lambda previous: previous == {'runId': 7, 'attempt': 1})
        second.upload(lambda row: bodies[row['name']], writer)
        self.assertEqual(registry.writes, list(bodies))
        self.assertEqual(registry.latest, {row['name']: row['version'] for row in manifest['packages']})
        before = list(registry.writes), list(registry.tags)
        second.upload(lambda row: bodies[row['name']], writer)
        self.assertEqual((registry.writes, registry.tags), before)
        self.assertEqual(store.read()[1]['active']['manifest'], manifest)

    def test_full_preflight_conflict_and_tampered_archives_write_nothing(self):
        store, manifest, bodies, registry, writer, runner = self.fixture()
        row = manifest['packages'][-1]; registry.values[row['name']] = b'conflicting bytes'
        current = runner(1); current.acquire()
        with self.assertRaisesRegex(ValueError, 'different bytes'):
            current.upload(lambda row: bodies[row['name']], writer)
        self.assertEqual(registry.writes, []); self.assertEqual(registry.tags, [])
        registry.values.clear()
        with self.assertRaisesRegex(ValueError, 'Original sealed archive differs'):
            current.upload(lambda row: b'tampered', writer)
        self.assertEqual(registry.writes, [])

    def test_latest_advancing_after_preflight_blocks_before_any_upload(self):
        store, manifest, bodies, registry, writer, runner = self.fixture()
        first = manifest['packages'][0]['name']; reads = 0
        def hook(name, selector):
            nonlocal reads
            if name == first and selector == '2.0.0':
                reads += 1
                if reads == 2: registry.latest[first] = '2.1.0-rc.1'
        registry.hook = hook
        current = runner(1); current.acquire()
        with self.assertRaisesRegex(ValueError, 'backwards'):
            current.upload(lambda row: bodies[row['name']], writer)
        self.assertEqual(registry.writes, []); self.assertEqual(registry.tags, [])

    def test_competing_same_version_rechecks_bytes_before_upload(self):
        for identical in [True, False]:
            with self.subTest(identical=identical):
                store, manifest, bodies, registry, writer, runner = self.fixture()
                first = manifest['packages'][0]['name']; reads = 0
                def hook(name, selector):
                    nonlocal reads
                    if name == first and selector == '2.0.0':
                        reads += 1
                        if reads == 2: registry.values[first] = bodies[first] if identical else b'other producer bytes'
                registry.hook = hook
                current = runner(1); current.acquire()
                if identical:
                    current.upload(lambda row: bodies[row['name']], writer)
                    self.assertNotIn(first, registry.writes)
                    self.assertEqual(registry.latest[first], '2.0.0')
                else:
                    with self.assertRaisesRegex(ValueError, 'different bytes'):
                        current.upload(lambda row: bodies[row['name']], writer)
                    self.assertEqual(registry.writes, [])


class FullInventoryBudgetContracts(unittest.TestCase):
    def test_297_package_publisher_preserves_preflight_fences_and_stage_budget(self):
        shared = module.load('budget_inventory_state_fixture', 'incremental-publication-state.test.py')
        api, store, manifest, _, _ = shared.fixture()
        now = [0.0]; reads = [0]; values = {}; writes = []; requests = []; latest = {}
        original_read = store.read
        def read():
            now[0] += 1; reads[0] += 1  # external ownership work, not registry I/O
            return original_read()
        store.read = read
        body = b'original frozen archive model'
        manifest['packages'] = [{'kind': 'npm', 'name': 'package-' + str(i), 'version': '2.0.0',
            'archive': str(i) + '.tgz', 'archiveBytes': len(body), 'archiveSha256': hashlib.sha256(body).hexdigest()}
            for i in range(297)]
        class Opener:
            def open(self, request, timeout):
                self_url = urllib.parse.urlsplit(request.full_url)
                name, selector = self_url.path.strip('/').split('/')
                requests.append((name, selector)); now[0] += 0.1
                if selector == 'latest': value = {'name': name, 'version': latest.get(name, '2.0.0-rc.1')}
                elif name not in values:
                    raise urllib.error.HTTPError(request.full_url, 404, 'absent', {}, None)
                elif selector == 'archive.tgz': return io.BytesIO(values[name])
                else:
                    value = {'name': name, 'version': '2.0.0', 'dist': {
                        'tarball': 'https://registry.npmjs.org/' + name + '/archive.tgz',
                        'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(values[name]).digest()).decode()}}
                return io.BytesIO(json.dumps(value).encode())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for row in manifest['packages']: (root / row['archive']).write_bytes(body)
            def command(argv, **options):
                if '--input-type=module' in argv:
                    return SimpleNamespace(stdout=str(Path(__file__).resolve()), returncode=0)
                if 'dist-tag' in argv:
                    name, version = argv[argv.index('add') + 1].rsplit('@', 1)
                    latest[name] = version
                    return SimpleNamespace(returncode=0)
                self.assertIn('publish', argv)
                self.assertGreaterEqual(len(requests), 594)  # entire first preflight precedes every write
                row = manifest['packages'][int(Path(argv[argv.index('publish') + 1]).stem)]
                self.assertEqual(original_read()[1]['active']['packages']['npm:' + row['name']], 'upload-started')
                values[row['name']] = body; writes.append(row['name'])
                return SimpleNamespace(returncode=0)
            writer = module.RegistryWriter({'GITHUB_ACTIONS': 'true', 'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://oidc.invalid/', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'oidc-fixture', 'NPM_TOKEN': 'fixture-only'}, root, command)
            writer.validate_archive = mock.Mock()  # synthetic bytes in registry-model tests
            self.addCleanup(writer.close)
            writer.reader = module.entry.evidence.registry_module.RegistryReadOnly(
                {'GITHUB_ACTIONS': 'true'}, Opener(), lambda: now[0])
            now[0] = 1000  # completed prior preparation cannot consume a later stage
            runner = module.state.Publication(store, manifest, {'runId': 7, 'attempt': 1}, lambda value: True)
            runner.acquire()
            delayed = [False]
            def slow_hash(row):
                if not delayed[0]: now[0] += 4501; delayed[0] = True
                return body
            with self.assertRaisesRegex(ValueError, 'deadline'):
                with writer.reader.budget(4500): runner.upload(slow_hash, writer)
            self.assertEqual(writes, [])
            self.assertEqual(requests, [])
            with writer.reader.budget(4500):
                runner.upload(lambda row: body, writer)
            self.assertEqual(len(writes), 297)
            self.assertGreater(now[0], 1600)
            self.assertEqual(set(original_read()[1]['active']['packages'].values()), {'registry-confirmed'})
            self.assertGreater(reads[0], 297)
            self.assertEqual(writer.reader.total_bytes, 297 * len(body))
            self.assertEqual(len(latest), 297)
            if hasattr(self, 'checkpoint'): self.checkpoint('fresh', reads[0], len(requests))
            # Reuse the same reader: cumulative bytes persist, but a new bounded
            # verification stage is not aged by an unrelated wait.
            now[0] += 1000
            with writer.reader.budget(4500):
                runner.upload(lambda row: body, writer)
            self.assertEqual(len(writes), 297)
            self.assertEqual(writer.reader.total_bytes, 3 * 297 * len(body))
            if hasattr(self, 'checkpoint'): self.checkpoint('reuse', reads[0], len(requests))


if __name__ == '__main__':
    unittest.main(verbosity=2)
