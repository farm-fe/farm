"""Actual-stage result recording contracts with a read-only fake registry."""
# cspell:words importlib
import base64
import io
import json
import urllib.parse
import copy
import importlib.util
import hashlib
import tempfile
import zipfile
from pathlib import Path
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("release_record_contract", Path(__file__).with_name("incremental-release-record.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


class Contracts(unittest.TestCase):
    def fixture(self):
        manifest = {"sourceTree": "b" * 40, "planHash": "c" * 64, "packages": [{"kind": "npm", "name": "a", "version": "2.0.0", "archiveSha256": "d" * 64}]}
        report = {"status": "passed", "publicRegistry": True, "workspacePatches": False, "cleanupConfirmed": True,
                  "stages": ["npm-install", "esm-cjs", "types", "docs-build", "sdk"]}
        reports = {platform: copy.deepcopy(report) for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]}
        registry = mock.MagicMock(); registry.verify.return_value = {"npm:a": "d" * 64}
        registry.json.return_value = {"name": "a", "version": "2.0.0"}
        return manifest, reports, registry

    def test_byte_integrity_does_not_claim_or_require_tests(self):
        manifest,_,registry=self.fixture()
        manifest.update(sourceSHA='a'*40,chain='release',namespace='farm-release-v1',releaseValidation='archive-integrity-v1')
        body=b'original sealed archive fixture'
        row=manifest['packages'][0]
        row.update(archive='a.tgz',archiveBytes=len(body),archiveSha256=hashlib.sha256(body).hexdigest())
        registry.verify.return_value={'npm:a':row['archiveSha256']}
        with tempfile.TemporaryDirectory() as directory:
            archives=Path(directory)/'archives';archives.mkdir();file=archives/'a.tgz';file.write_bytes(body)
            self.assertIsNone(module.check_archives(manifest,directory,'a'*40))
            proof=module.registry_record(manifest,{'id':12},registry)
            self.assertTrue(module.entry.public_integrity_verified(proof,manifest))
            self.assertFalse(proof['registryInstall']);self.assertFalse(proof['nativeABI']);self.assertFalse(proof['testsExecuted'])
            self.assertNotIn('scratchRemoved',proof);self.assertNotIn('cleanupConfirmed',proof)
            for field,value in [('testsExecuted',True),('registryBytesVerified',False),('registryInstall',True),('latestTargets',False)]:
                self.assertFalse(module.entry.public_integrity_verified({**proof,field:value},manifest))
            with self.assertRaisesRegex(ValueError,'source differs'):module.check_archives(manifest,directory,'f'*40)
            registry.verify.return_value={'npm:a':'0'*64}
            with self.assertRaisesRegex(ValueError,'registry bytes'):module.registry_record(manifest,{'id':12},registry)
            file.write_bytes(body.replace(b'original',b'modified'))
            with self.assertRaisesRegex(ValueError,'bytes differ'):module.check_archives(manifest,directory,'a'*40)
            file.unlink()
            with self.assertRaises(FileNotFoundError):module.check_archives(manifest,directory,'a'*40)

    def test_frozen_manifest_alone_selects_old_or_new_proof_semantics(self):
        legacy,reports,registry=self.fixture();current={**legacy,'releaseValidation':'archive-integrity-v1'}
        old=module.public_record(legacy,reports,{'id':12},registry)
        new=module.registry_record(current,{'id':12},registry)
        self.assertTrue(module.entry.public_integrity_verified(old,legacy))
        self.assertTrue(module.entry.public_integrity_verified(new,current))
        self.assertFalse(module.entry.public_integrity_verified(new,legacy))
        self.assertFalse(module.entry.public_integrity_verified(old,current))
        with self.assertRaisesRegex(ValueError,'policy'):module.public_record(current,reports,{'id':12},registry)
        with self.assertRaisesRegex(ValueError,'policy'):module.registry_record(legacy,{'id':12},registry)
        reports['windows-latest']['kind']='release-byte-integrity-v1'
        with self.assertRaisesRegex(ValueError,'byte-only'):module.public_record(legacy,reports,{'id':12},registry)

    def test_public_proof_requires_actual_stages_all_platforms_registry_bytes_and_latest(self):
        manifest, reports, registry = self.fixture()
        result = module.public_record(manifest, reports, {"id": 12}, registry)
        self.assertEqual(result["packages"], {"npm:a": "d" * 64})
        self.assertTrue(result["latestTargets"]); registry.verify.assert_called_once()

    def test_missing_platform_or_unconfirmed_cleanup_blocks_proof(self):
        manifest, reports, registry = self.fixture(); reports.pop("macos-latest")
        with self.assertRaisesRegex(ValueError, "three public"): module.public_record(manifest, reports, {"id": 12}, registry)
        manifest, reports, registry = self.fixture(); reports["windows-latest"]["cleanupConfirmed"] = False
        with self.assertRaisesRegex(ValueError, "cleanup failed"): module.public_record(manifest, reports, {"id": 12}, registry)
        registry.verify.assert_not_called()

    def test_missing_sdk_or_different_registry_byte_digest_cannot_be_reported_passed(self):
        manifest, reports, registry = self.fixture(); reports["ubuntu-24.04"]["stages"].remove("sdk")
        with self.assertRaisesRegex(ValueError, "stages missing"): module.public_record(manifest, reports, {"id": 12}, registry)
        manifest, reports, registry = self.fixture(); registry.verify.return_value = {"npm:a": "e" * 64}
        with self.assertRaisesRegex(ValueError, "bytes differ"): module.public_record(manifest, reports, {"id": 12}, registry)

    def test_newer_latest_is_preserved_but_older_or_other_package_rejected(self):
        manifest, reports, registry = self.fixture(); registry.json.return_value = {"name": "a", "version": "2.1.0"}
        self.assertTrue(module.public_record(manifest, reports, {"id": 12}, registry)["latestTargets"])
        for value in [{"name": "a", "version": "1.9.0"}, {"name": "other", "version": "2.0.0"}]:
            registry.json.return_value = value
            with self.assertRaises(ValueError): module.public_record(manifest, reports, {"id": 12}, registry)


class InventoryBudgetContracts(unittest.TestCase):
    def fixture(self, seconds=1):
        now = [1000.0]; calls = []
        body = b'frozen production-algorithm test bytes'
        rows = [{'kind': 'npm', 'name': 'package-' + str(i), 'version': '2.0.0',
                 'archiveSha256': hashlib.sha256(body).hexdigest()} for i in range(297)]
        class Opener:
            def open(self, request, timeout):
                calls.append(request.full_url); now[0] += seconds
                name, selector = urllib.parse.urlsplit(request.full_url).path.strip('/').split('/')
                if selector == 'archive.tgz': value = body
                else:
                    value = json.dumps({'name': name, 'version': '2.0.0', 'dist': {
                        'tarball': 'https://registry.npmjs.org/' + name + '/archive.tgz',
                        'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode()}}).encode()
                return io.BytesIO(value)
        client = module.entry.evidence.registry_module.RegistryReadOnly({'GITHUB_ACTIONS': 'true'}, Opener(), lambda: now[0])
        manifest = {'releaseValidation': 'archive-integrity-v1', 'sourceTree': 'b' * 40, 'planHash': 'c' * 64, 'packages': rows}
        now[0] += 1000  # External source/lock work must not age the reader.
        return manifest, client, now, calls

    def test_full_public_proof_covers_archives_and_latest_with_one_batch_deadline(self):
        manifest, client, now, calls = self.fixture()
        result = module.registry_record(manifest, {'id': 7}, client)
        self.assertEqual(len(result['packages']), 297)
        self.assertEqual(len(calls), 891)
        self.assertEqual(now[0], 2891)
        self.assertTrue(result['registryBytesVerified'])
        self.assertFalse(result['testsExecuted'])

    def test_latest_phase_cannot_restart_exhausted_public_batch(self):
        manifest, client, _, calls = self.fixture(seconds=2)
        with self.assertRaisesRegex(ValueError, 'deadline'):
            module.registry_record(manifest, {'id': 7}, client)
        self.assertGreater(len(calls), 594)  # archive loop passed; latest uses same budget
        self.assertLess(len(calls), 891)

    def test_finalize_style_full_inventory_and_recovery_retain_byte_accounting(self):
        manifest, client, now, calls = self.fixture(seconds=2)
        records = {'npm:' + row['name']: row for row in manifest['packages']}
        expected = {key: row['archiveSha256'] for key, row in records.items()}
        self.assertEqual(client.verify(records), expected)
        count = client.total_bytes
        now[0] += 1000
        self.assertEqual(client.verify(records), expected)
        self.assertEqual(client.total_bytes, 2 * count)
        self.assertEqual(len(calls), 1188)



class PresenceRecordContracts(unittest.TestCase):
    def setUp(self):
        self.bodies = {'a': b'new source a', 'b': b'existing public b', 'c': b'existing race c'}
        self.metadata = {name: {field: {} for field in module.selector.METADATA_FIELDS} for name in self.bodies}
        self.family = [{'kind': 'npm', 'name': name, 'version': '2.0.0', 'manifestPath': 'packages/' + name + '/package.json'}
                       for name in self.bodies]
        self.latest = {}
        self.manifest = {'sourceSHA': 'a' * 40, 'sourceTree': 'b' * 40, 'planHash': 'c' * 64,
            'chain': 'release', 'profile': 'release', 'namespace': 'farm-release-v1', 'publicationScope': 'npm',
            'releaseValidation': 'archive-integrity-v1', 'selectionPolicy': 'registry-exact-version-v2', 'baseline': None,
            'producerRunId': 10, 'producerAttempt': 1, 'family': self.family,
            'packages': [self.candidate('a', self.bodies['a']), self.candidate('c', b'new source c differs from race')],
            'publish': ['npm:a', 'npm:c'], 'rebuild': ['npm:a', 'npm:c'], 'buildFailures': {}}
        self.manifest['registrySnapshot'] = {'schemaVersion': 2, 'sourceSHA': 'a' * 40, 'sourceTree': 'b' * 40,
            'packages': {'npm:' + name: self.observed(name) if name == 'b' else {'status': 'missing', 'name': name, 'version': '2.0.0'}
                         for name in self.bodies}}
        self.report = self.completed_report()
        self.environment = {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '2', 'GITHUB_SHA': 'a' * 40,
                            'EXPECTED_PUBLISHER_RESULT_ARTIFACT_ID': '900'}
        self.repository = {'id': 5, 'default_branch': 'main'}
        self.run = {'id': 10, 'run_attempt': 2, 'head_sha': 'a' * 40, 'head_branch': 'main', 'event': 'push',
                    'path': module.entry.WORKFLOW, 'status': 'in_progress', 'repository': {'id': 5}, 'head_repository': {'id': 5}}
        self.job = {'id': 100, 'name': module.entry.PUBLISHER, 'run_id': 10, 'run_attempt': 2, 'head_sha': 'a' * 40,
                    'status': 'completed', 'conclusion': 'success', 'steps': [
                        {'name': module.history.UPLOAD_STEP, 'status': 'completed', 'conclusion': 'success'}]}
        self.artifact = {'id': 900, 'name': 'npm-publication-results-10-2', 'expired': False,
                         'workflow_run': {'id': 10, 'repository_id': 5, 'head_repository_id': 5, 'head_sha': 'a' * 40}}
        self.refresh_archive()
        self.transport = mock.Mock()
        self.transport.get.side_effect = self.get
        self.transport.archive.side_effect = lambda identity: self.archive if identity == 900 else self.fail('Wrong artifact ID')

    def candidate(self, name, body):
        return {'kind': 'npm', 'name': name, 'version': '2.0.0', 'archive': name + '.tgz',
                'archiveSha256': hashlib.sha256(body).hexdigest(), 'archiveBytes': len(body)}

    def observed(self, name, *, verify_archive=False):
        body = self.bodies[name]
        return {'status': 'published', 'name': name, 'version': '2.0.0',
                **({'archiveSha256': hashlib.sha256(body).hexdigest(), 'archiveBytes': len(body)} if verify_archive else {}),
                'evidenceKind': 'registry-archive' if verify_archive else 'registry-metadata',
                'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode(),
                'metadata': copy.deepcopy(self.metadata[name]), 'tarballURL': 'https://registry.npmjs.org/' + name + '/archive.tgz'}

    def completed_report(self):
        report = module.best_effort.new_report(self.manifest)
        report.update(runId=10, attempt=2, complete=True, success=True)
        origins = {'a': 'source', 'b': 'registry', 'c': 'registry-race'}
        for row in report['packages']:
            name = row['name']
            row.update(self.observed(name, verify_archive=name == 'a'))
            row.update(status='success', originKind=origins.get(name, 'registry'), uploadAttempted=name == 'a',
                       outcomeUncertain=False, latestOutcomeUncertain=False, registryMetadataVerified=True, registryBytesVerified=name == 'a', latestVerified=True, errors=[])
        count = len(report['packages']); published = sum(row['uploadAttempted'] for row in report['packages'])
        report['counts'] = {'total': count, 'success': count, 'failed': 0, 'published': published, 'retained': count - published}
        return report

    def refresh_archive(self, *, entries=None):
        self.local_bytes = json.dumps(self.report, sort_keys=True).encode() + b'\n'
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
            for name, body in entries or [('results.json', self.local_bytes)]:
                archive.writestr(name, body)
        self.archive = buffer.getvalue()
        digest = hashlib.sha256(self.archive).hexdigest()
        self.artifact.update(size_in_bytes=len(self.archive), digest='sha256:' + digest)
        self.environment['EXPECTED_PUBLISHER_RESULT_ARTIFACT_DIGEST'] = digest

    def get(self, route):
        if route == '': return self.repository
        if route == '/actions/runs/10/attempts/2': return self.run
        if route == '/actions/runs/10/attempts/2/jobs?per_page=100&page=1': return {'jobs': [self.job]}
        if route == '/actions/artifacts/900': return self.artifact
        raise AssertionError(route)

    def authenticate(self, local=None):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / 'results.json'; file.write_bytes(self.local_bytes if local is None else local)
            return module.authenticated_publisher_report(self.transport, self.manifest, file, self.environment)

    def reader(self, seconds=1):
        bodies, metadata, latest = self.bodies, self.metadata, self.latest
        now, calls = [1000.0], []
        class Response(io.BytesIO):
            def getcode(self): return 200
        class Opener:
            def open(self, request, timeout):
                calls.append(request.full_url); now[0] += seconds
                name, selection = urllib.parse.urlsplit(request.full_url).path.strip('/').split('/')
                if selection == 'archive.tgz': return Response(bodies[name])
                if selection == 'latest': return Response(json.dumps(latest.get(name, {'name': name, 'version': '2.0.0'})).encode())
                body = bodies[name]
                return Response(json.dumps({'name': name, 'version': '2.0.0', **metadata[name],
                    'dist': {'tarball': 'https://registry.npmjs.org/' + name + '/archive.tgz',
                             'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode()}}).encode())
        return module.selector.ExactRegistryReadOnly({'GITHUB_ACTIONS': 'true'}, Opener(), lambda: now[0]), now, calls

    def proof(self):
        reader, _, calls = self.reader()
        return module.presence_registry_record(self.manifest, {'id': 12}, reader, self.report), calls

    def test_new_retained_and_race_proof_keeps_independent_origins(self):
        report, binding = self.authenticate()
        self.assertEqual(report, self.report)
        self.assertEqual(binding, {'artifactId': 900, 'artifactDigest': self.artifact['digest'], 'runId': 10,
                                   'attempt': 2, 'publisherJobId': 100})
        proof, calls = self.proof()
        self.assertEqual(proof['kind'], 'release-version-presence-v1')
        self.assertEqual(len(calls), 7)
        self.assertEqual(sum(url.endswith('/archive.tgz') for url in calls), 1)
        self.assertEqual(proof['dispositions']['npm:a']['originKind'], 'source')
        self.assertEqual(proof['dispositions']['npm:b']['originKind'], 'registry')
        self.assertEqual(proof['dispositions']['npm:c']['originKind'], 'registry-race')
        self.assertIsNone(proof['packages']['npm:c']); self.assertIsNone(proof['packages']['npm:b'])
        self.assertNotIn('archiveSha256', proof['dispositions']['npm:b']); self.assertNotIn('archiveBytes', proof['dispositions']['npm:c'])
        self.assertFalse(proof['testsExecuted']); self.assertFalse(proof['nativeABI']); self.assertFalse(proof['registryInstall'])
        self.assertNotIn('registryBytesVerified', proof); self.assertTrue(proof['registryMetadataVerified']); self.assertTrue(proof['latestTargets'])

    def test_empty_observation_archives_allowed_only_for_v2(self):
        self.family[:] = [row for row in self.family if row['name'] == 'b']
        self.manifest.update(packages=[], publish=[], rebuild=[])
        self.manifest['registrySnapshot']['packages'] = {'npm:b': self.observed('b')}
        self.report = self.completed_report(); self.refresh_archive()
        with tempfile.TemporaryDirectory() as directory:
            module.check_archives(self.manifest, directory, 'a' * 40)
            proof, calls = self.proof(); self.assertEqual(proof['packages'], {'npm:b': None}); self.assertEqual(len(calls), 2)
            self.assertFalse(any(url.endswith('/archive.tgz') for url in calls))
            legacy = {key: value for key, value in self.manifest.items() if key not in ['registrySnapshot', 'selectionPolicy']}
            with self.assertRaisesRegex(ValueError, 'inventory missing'): module.check_archives(legacy, directory, 'a' * 40)

    def test_publisher_report_requires_every_success_and_exact_identity(self):
        cases = [('schemaVersion', 1), ('complete', False), ('success', False), ('fatal', {'stage': 'failure'}),
                 ('sourceSHA', 'f' * 40), ('sourceTree', 'f' * 40), ('manifestHash', 'f' * 64),
                 ('runId', 11), ('attempt', 1), ('producerRunId', 11), ('producerAttempt', 2),
                 ('baselineAdvanced', True), ('releaseCreatedByThisStage', True), ('packages', self.report['packages'][:-1])]
        for field, value in cases:
            with self.subTest(field=field):
                report = copy.deepcopy(self.report); report[field] = value
                with self.assertRaises(ValueError): module.validate_publisher_report(self.manifest, report, 10, 2)
        for field, value in [('status', 'failed'), ('errors', [{'stage': 'publish'}]), ('latestVerified', False),
                             ('registryBytesVerified', False), ('outcomeUncertain', True), ('latestOutcomeUncertain', True)]:
            with self.subTest(field=field):
                report = copy.deepcopy(self.report); report['packages'][0][field] = value
                with self.assertRaises(ValueError): module.validate_publisher_report(self.manifest, report, 10, 2)

    def test_artifact_and_job_binding_cannot_be_forged_by_report(self):
        for target, changes in [(self.run, {'head_sha': 'f' * 40}), (self.run, {'run_attempt': 1}),
                (self.run, {'event': 'workflow_dispatch'}), (self.job, {'conclusion': 'failure'}),
                (self.job, {'run_attempt': 1}), (self.artifact, {'name': 'npm-publication-results-10-1'}),
                (self.artifact, {'expired': True}), (self.artifact, {'id': 901}),
                (self.artifact, {'digest': 'sha256:' + 'f' * 64}),
                (self.artifact, {'workflow_run': {**self.artifact['workflow_run'], 'head_sha': 'f' * 40}}),
                (self.environment, {'EXPECTED_PUBLISHER_RESULT_ARTIFACT_DIGEST': ''})]:
            with self.subTest(changes=changes):
                prior = copy.deepcopy(target); target.update(changes)
                with self.assertRaises(ValueError): self.authenticate()
                target.clear(); target.update(prior)
        self.job['steps'][0]['conclusion'] = 'failure'
        with self.assertRaisesRegex(ValueError, 'upload did not succeed'): self.authenticate()

    def test_local_bytes_and_bounded_unique_artifact_members_are_required(self):
        with self.assertRaisesRegex(ValueError, 'Local publisher results'): self.authenticate(self.local_bytes + b' ')
        for entries in [[('../results.json', self.local_bytes)], [('results.json', self.local_bytes), ('other', b'x')],
                        [('results.json', b'{"duplicate":1,"duplicate":2}')]]:
            self.refresh_archive(entries=entries)
            with self.assertRaises(ValueError): self.authenticate()
        self.refresh_archive(); self.archive = self.archive + b'tampered'
        with self.assertRaisesRegex(ValueError, 'digest differs'): self.authenticate()

    def test_unknown_or_build_failure_cannot_yield_complete_proof(self):
        self.manifest['registrySnapshot']['packages']['npm:b'] = {'status': 'unknown', 'name': 'b', 'version': '2.0.0', 'reason': 'registry-unavailable'}
        with self.assertRaisesRegex(ValueError, 'unknown versions'): self.proof()
        self.setUp(); self.manifest['packages'].pop(); self.manifest['buildFailures'] = {'npm:c': 'native-build-failed'}
        with self.assertRaisesRegex(ValueError, 'build failures'): self.proof()

    def test_public_record_does_not_open_unused_corrupt_or_missing_race_candidate(self):
        with tempfile.TemporaryDirectory() as directory:
            archives = Path(directory) / 'archives'; archives.mkdir()
            (archives / 'a.tgz').write_bytes(self.bodies['a'])
            # c's old candidate is not the version the publisher retained.
            module.check_archives(self.manifest, directory, 'a' * 40, publisher_report=self.report)
            (archives / 'c.tgz').write_bytes(b'corrupt unused race candidate')
            module.check_archives(self.manifest, directory, 'a' * 40, publisher_report=self.report)
            (archives / 'a.tgz').write_bytes(b'corrupt source upload')
            with self.assertRaises(ValueError):
                module.check_archives(self.manifest, directory, 'a' * 40, publisher_report=self.report)

    def test_source_cannot_become_retained_and_retained_cannot_become_source(self):
        for name, origin, attempted in [('a', 'source', False), ('b', 'source', True), ('c', 'registry-race', True)]:
            with self.subTest(name=name):
                original = copy.deepcopy(self.report)
                row = next(row for row in self.report['packages'] if row['name'] == name)
                row.update(originKind=origin, uploadAttempted=attempted)
                published = sum(row['uploadAttempted'] for row in self.report['packages'])
                self.report['counts'].update(published=published, retained=3 - published)
                with self.assertRaises(ValueError): self.proof()
                self.report = original

    def test_changed_actual_metadata_or_retained_observation_is_rejected(self):
        self.metadata['b']['dependencies'] = {'a': '2.0.0'}
        with self.assertRaisesRegex(ValueError, 'changed since publisher'): self.proof()
        self.report = self.completed_report()
        with self.assertRaisesRegex(ValueError, 'Retained registry observation changed'): self.proof()
        self.setUp(); self.bodies['a'] = b'NEW SOURCE A'; self.report = self.completed_report()
        with self.assertRaisesRegex(ValueError, 'New source archive origin differs'): self.proof()

    def test_public_dependency_peer_ranges_and_latest_are_rechecked(self):
        self.latest = {name: {'name': name, 'version': '2.9.0'} for name in self.bodies}
        self.assertTrue(self.proof()[0]['latestTargets'])
        for field in ['dependencies', 'peerDependencies']:
            self.metadata['c'][field] = {'a': '^3.0.0'}; self.report = self.completed_report()
            with self.assertRaisesRegex(ValueError, 'dependency incompatible'): self.proof()
            self.metadata['c'][field] = {}
        self.report = self.completed_report(); self.latest['a'] = {'name': 'a', 'version': '1.0.0'}
        with self.assertRaisesRegex(ValueError, 'behind'): self.proof()
        self.latest['a'] = {'name': 'other', 'version': '2.9.0'}
        with self.assertRaisesRegex(ValueError, 'identity differs'): self.proof()

    def test_large_source_family_byte_limit_comes_from_frozen_inventory_not_default_cap(self):
        count, size = 297, 16 * 1024 ** 2
        self.bodies = {'package-' + str(index): b'archive' for index in range(count)}
        self.metadata = {name: {field: {} for field in module.selector.METADATA_FIELDS} for name in self.bodies}
        self.family[:] = [{'kind': 'npm', 'name': name, 'version': '2.0.0', 'manifestPath': 'packages/' + name + '/package.json'} for name in self.bodies]
        self.manifest.update(packages=[{**self.candidate(name, body), 'archiveBytes': size} for name, body in self.bodies.items()],
                             publish=['npm:' + name for name in self.bodies], rebuild=['npm:' + name for name in self.bodies])
        self.manifest['registrySnapshot']['packages'] = {'npm:' + name: {'status': 'missing', 'name': name, 'version': '2.0.0'} for name in self.bodies}
        self.report = self.completed_report()
        for row in self.report['packages']:
            row.update(self.observed(row['name'], verify_archive=True))
            row.update(status='success', originKind='source', uploadAttempted=True, registryBytesVerified=True, archiveBytes=size)
        self.report['counts'].update(published=count, retained=0)
        reader, _, _ = self.reader()
        def measured_archive(url):
            name = urllib.parse.urlsplit(url).path.strip('/').split('/')[0]
            body = self.bodies[name]; reader.total_bytes += size
            self.assertLessEqual(reader.total_bytes, reader.total_byte_limit)
            return {'sha256': hashlib.sha256(body).hexdigest(), 'sha512': base64.b64encode(hashlib.sha512(body).digest()).decode(),
                    'sha1': hashlib.sha1(body).hexdigest(), 'bytes': size}
        # Exercise the production budget calculation with measured fixture sizes,
        # without allocating gigabytes or fetching any production archive.
        with mock.patch.object(reader, 'archive', side_effect=measured_archive):
            result = module.presence_registry_record(self.manifest, {'id': 12}, reader, self.report)
        self.assertEqual(len(result['packages']), count)
        self.assertEqual(reader.total_byte_limit, size * count); self.assertGreater(reader.total_byte_limit, 3 * 1024 ** 3)

    def test_full_297_retained_family_never_downloads_archives_and_shares_one_deadline(self):
        self.bodies = {'package-' + str(index): b'archive' for index in range(297)}
        self.metadata = {name: {field: {} for field in module.selector.METADATA_FIELDS} for name in self.bodies}
        self.family[:] = [{'kind': 'npm', 'name': name, 'version': '2.0.0', 'manifestPath': 'packages/' + name + '/package.json'} for name in self.bodies]
        self.manifest.update(packages=[], publish=[], rebuild=[])
        self.manifest['registrySnapshot']['packages'] = {'npm:' + name: self.observed(name) for name in self.bodies}
        self.report = self.completed_report()
        reader, now, calls = self.reader()
        result = module.presence_registry_record(self.manifest, {'id': 12}, reader, self.report)
        self.assertEqual(len(result['packages']), 297); self.assertEqual(len(calls), 594)
        self.assertEqual(reader.total_bytes, 0); self.assertEqual(reader.total_byte_limit, 1)
        self.assertFalse(any(url.endswith('/archive.tgz') for url in calls))
        self.assertTrue(all(value is None for value in result['packages'].values()))
        reader, _, calls = self.reader(seconds=3)
        with self.assertRaisesRegex(ValueError, 'deadline'):
            module.presence_registry_record(self.manifest, {'id': 12}, reader, self.report)
        self.assertGreater(len(calls), 297); self.assertLess(len(calls), 594)

if __name__ == "__main__":
    unittest.main(verbosity=2)
