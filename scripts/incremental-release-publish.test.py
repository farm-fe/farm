"""Pure HTTP/Release finalization contracts; no real requests or registry writes."""
# cspell:words importlib
import copy
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
        verifier = mock.Mock(); verifier.verify.return_value = {'verified': True}
        owner = {'runId': 10, 'attempt': 2}
        valid_body = api.release['body']
        with mock.patch.object(module.entry.evidence, 'source_reader', return_value=fixture.source):
            for mutation in ['missing', 'corrupt', 'valid']:
                with self.subTest(checkpoint=mutation):
                    api.release['body'] = valid_body
                    if mutation == 'missing': api.release['body'] = '<!-- farm-release-manifest:' + transaction + ' -->'
                    if mutation == 'corrupt': api.release['body'] = valid_body.replace('farm-release-checkpoint-v1:', 'farm-release-checkpoint-v1:' + '0')
                    retry = module.state.Publication(store, manifest, owner, lambda value: True)
                    verify = lambda receipt: module.completed_publication_receipt(api, manifest, verifier, Path('.'), owner)
                    if mutation == 'valid':
                        retry.recover(lambda previous: True, verify)
                        self.assertTrue(retry.completed)
                    else:
                        with self.assertRaisesRegex(ValueError, 'checkpoint missing or ambiguous'):
                            retry.recover(lambda previous: True, verify)
                        self.assertFalse(retry.completed)
                    self.assertEqual(api.writes, [])
            self.assertEqual(verifier.verify.call_count, 3)

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
        reader=mock.Mock();reader.json.side_effect=metadata;reader.archive.side_effect=bodies
        with mock.patch.object(module.entry.evidence.registry_module,'RegistryReadOnly',return_value=reader):
            writer=module.RegistryWriter({},'.')
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
        fixture.manifest['producerAttempt']=producer_attempt
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
            fixture.jobs_by_attempt[attempt].append(job);candidate_jobs.append(job)
        fixture.jobs=base_jobs+candidate_jobs
        archive(61,'farm-release-seal-'+fixture.source['sha']+'-'+str(producer_attempt),fixture.seal)
        archive(60,'farm-release-public-acceptance-'+fixture.source['sha']+'-'+str(current_attempt),fixture.public)
        def get(route):
            if route=='':return fixture.repository
            if route=='/branches/main':return {'commit':{'sha':fixture.source['sha']}}
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
        verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':current_attempt},lambda sha:fixture.source)
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
        with mock.patch.object(module.entry.execution,'jobs_for',return_value=fixture.jobs), mock.patch.object(module.entry.execution,'fetch_json_artifact',side_effect=[(fixture.seal,fixture.artifact),(fixture.public,fixture.artifact)]), mock.patch.object(module.entry.evidence.registry_module,'RegistryReadOnly',return_value=registry), mock.patch.object(module,'verify_public_selection',return_value=[]):
            proof=verifier.verify(fixture.manifest,fixture.root,phase='postpublish')
        self.assertTrue(proof['verified']);self.assertEqual(proof['producerAttempt'],2);self.assertEqual(proof['publicAttempt'],3)
        registry.verify.assert_called_once_with({row['kind']+':'+row['name']:row for row in fixture.manifest['packages']})

    def test_workflow_joint_contract_keeps_source_producer_attempt_and_explicit_writes(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        self.assertIn('group: farm-independent-release\n  cancel-in-progress: false',workflow)
        for stage in ['publisher','public-consumers','public-proof','finalize']:
            block=re.search(r'^  '+re.escape(stage)+r':\n(.*?)(?=^  [A-Za-z][A-Za-z0-9-]*:\n|\Z)',workflow,re.M|re.S).group(1)
            self.assertIn('needs.source.outputs.producer_attempt',block)
        self.assertIn("inputs.publish == true",workflow)
        self.assertIn('name: '+module.ROLES['upload'],workflow)
        self.assertIn('name: '+module.ROLES['finalize'],workflow)
        self.assertIn('profile: release',workflow)
        self.assertIn('scripts/incremental-release-functional.py --source',workflow)
        retired=(Path(__file__).parents[1]/'.github/workflows/v2-stable-release.yaml').read_text(encoding='utf-8')
        self.assertNotIn('pull_request:',retired);self.assertNotIn('secrets.',retired);self.assertNotIn('v2-auto-release',retired)

    def test_latest_confirmation_preserves_higher_version_without_command(self):
        writer,reader=self.registry([{'name':'a','version':'3.0.0'}],[])
        writer.command=mock.Mock()
        writer.confirm({'kind':'npm','name':'a','version':'2.0.0'})
        writer.command.assert_not_called()

    def public_fixture(self):
        manifest={'chain':'release','profile':'release','sourceSHA':'a'*40,'sourceTree':'b'*40,'producerRunId':7,'producerAttempt':1,'planHash':'c'*64}
        runs={attempt:{'id':7,'run_attempt':attempt,'path':module.entry.WORKFLOW,'head_sha':manifest['sourceSHA'],
              'head_branch':'main','event':'workflow_dispatch','repository':{'id':5},'head_repository':{'id':5}}
              for attempt in [1,2]}
        jobs={1:[],2:[]};artifacts={};bodies={};reports={}
        for index,(platform,attempt) in enumerate([('ubuntu-24.04',1),('macos-latest',1),('windows-latest',2)],1):
            job={'id':index+10,'name':'Independent public registry consumer ('+platform+')','run_id':7,'run_attempt':attempt,
                 'status':'completed','conclusion':'success','steps':[{'name':module.PUBLIC_UPLOAD,'status':'completed','conclusion':'success',
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
        def get(route):
            if route=='':return {'id':5,'default_branch':'main'}
            if route=='/actions/runs/7/artifacts?per_page=100&page=1':return {'artifacts':list(artifacts.values())}
            if '/attempts/' in route:return runs[int(route.rsplit('/',1)[1])]
            if route.startswith('/actions/artifacts/'):return artifacts[int(route.rsplit('/',1)[1])]
            raise AssertionError('Unexpected public report API route: '+route)
        transport=mock.Mock();transport.get.side_effect=get;transport.archive.side_effect=lambda number:bodies[number]
        return manifest,transport,jobs,artifacts,bodies,reports,runs

    def test_mixed_attempt_public_matrix_is_bound_independently_for_each_os(self):
        manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture()
        output=Path(tempfile.mkdtemp(prefix='farm-public-mixed-contract-'))/'reports'
        with mock.patch.object(module.entry.execution,'jobs_for',side_effect=lambda client,run:jobs[run['run_attempt']]):
            selection=module.collect_public_reports(transport,manifest,{'runId':7,'attempt':2},output)
            verified=module.verify_public_selection(transport,manifest,7,2,selection['reports'])
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
            with self.assertRaisesRegex(ValueError,'platform binding differs'):
                module.verify_public_report(transport,manifest,7,2,'windows-latest',3)

    def test_wrong_public_source_or_artifact_digest_cannot_resume(self):
        for field,value in [('head_sha','f'*40),('event','pull_request'),('head_repository',{'id':99})]:
            manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture();runs[2][field]=value
            with self.assertRaisesRegex(ValueError,'main origin differs'):
                module.verify_public_report(transport,manifest,7,2,'windows-latest',3)
        manifest,transport,jobs,artifacts,bodies,reports,runs=self.public_fixture();artifacts[3]['digest']='sha256:'+'f'*64
        with mock.patch.object(module.entry.execution,'jobs_for',side_effect=lambda client,run:jobs[run['run_attempt']]):
            with self.assertRaisesRegex(ValueError,'digest differs'):module.verify_public_report(transport,manifest,7,2,'windows-latest',3)

    def frozen_fixture(self):
        fixture,verifier,original,current=self.recovery_fixture()
        spec=importlib.util.spec_from_file_location('state_frozen_fixture',Path(__file__).with_name('incremental-publication-state.test.py'))
        shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
        store=module.state.GitRefStore(shared.GitAPI())
        owner={'runId':10,'attempt':2,'repositoryId':5,'sourceSHA':fixture.source['sha'],'sourceTree':fixture.source['tree'],
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
        frozen=module.frozen_authorization(store,fixture.manifest,fixture.source,5,{'runId':10,'attempt':3})
        verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':3},lambda sha:newer if sha==newer['sha'] else fixture.source,frozen=frozen)
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
            module.frozen_authorization(store,fixture.manifest,fixture.source,5,{'runId':10,'attempt':3})
        for field,value in [('event','pull_request'),('path','.github/workflows/ci.yaml'),('head_repository',{'id':99})]:
            fixture,verifier,original,current,shared,store,owner,first=self.frozen_fixture()
            frozen=module.frozen_authorization(store,fixture.manifest,fixture.source,5,{'runId':10,'attempt':3})
            original[field]=value
            verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':3},lambda sha:fixture.source,frozen=frozen)
            with self.assertRaisesRegex(ValueError,'not terminal or its source differs'):verifier.verify(fixture.manifest,fixture.root)

    def test_public_matrix_has_no_shared_attempt_and_uses_explicit_cross_platform_shell(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        self.assertNotIn('consumer_attempt:',workflow);self.assertNotIn('outputs.consumer_attempt',workflow)
        block=workflow.split('      - name: Record actual consumer attempt',1)[1].split('      - name: Upload independent public consumer report',1)[0]
        self.assertIn('shell: python',block);self.assertIn('publicationBinding',block)
        self.assertIn('collect-public --candidate release-candidate',workflow)
        self.assertIn('bind-public-proof --candidate release-candidate',workflow)

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
        self.assertIn("needs.plan.outputs.publish_required == 'true'",producer)

    def test_candidate_success_in_intermediate_attempt_survives_a_third_attempt(self):
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        proof=verifier.verify(fixture.manifest,fixture.root)
        self.assertTrue(proof['prepublicationVerified'])
        self.assertEqual({platform:row['attempt'] for platform,row in proof['candidateReports'].items()},
                         {'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        baseline=module.entry.load('joint_candidate_baseline','incremental-release-baseline.py').ReleaseBaselineReader(fixture.transport,None,lambda sha:fixture.source)
        self.assertEqual(proof['candidateReports'],baseline.candidates(fixture.manifest,fixture.repository,3))

    def test_candidate_middle_attempt_rejects_wrong_source_and_wrong_host_report(self):
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        fixture.runs[2]['head_sha']='f'*40
        with self.assertRaisesRegex(ValueError,'Candidate acceptance source'):verifier.verify(fixture.manifest,fixture.root)
        fixture,verifier,original,current=self.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        report=json.loads(zipfile.ZipFile(io.BytesIO(fixture.candidate_bodies[200])).read('result.json'))
        report['platform']={'os':'win32','arch':'x64','abi':'win32-x64-msvc'}
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w') as file:file.writestr('result.json',json.dumps(report))
        body=output.getvalue();fixture.candidate_bodies[200]=body
        fixture.candidate_metadata[200].update({'size_in_bytes':len(body),'digest':'sha256:'+hashlib.sha256(body).hexdigest()})
        with self.assertRaisesRegex(ValueError,'No valid independent candidate acceptance|OS or ABI differs'):verifier.verify(fixture.manifest,fixture.root)

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

    def test_workflow_uploads_the_candidate_report_expected_by_the_baseline_reader(self):
        workflow=(Path(__file__).parents[1]/'.github/workflows/incremental-release.yaml').read_text(encoding='utf-8')
        block=workflow.split('  candidate-consumers:',1)[1].split('  publisher:',1)[0]
        self.assertIn('name: Upload independent release candidate acceptance',block)
        self.assertIn('name: farm-release-candidate-consumer-${{ github.sha }}-${{ matrix.os }}-${{ github.run_attempt }}',block)
        self.assertIn('path: candidate-results/result.json',block)

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
        fixture.transport.get.side_effect=history
        verifier=module.RecoveryVerifier(fixture.transport,{'runId':10,'attempt':3},lambda sha:newer if sha==newer['sha'] else fixture.source)
        self.assertTrue(verifier.verify(fixture.manifest,fixture.root)['prepublicationVerified'])
        comparison['status']='diverged'
        with self.assertRaisesRegex(ValueError,'trusted main history'):verifier.verify(fixture.manifest,fixture.root)


if __name__ == '__main__':
    unittest.main(verbosity=2)
