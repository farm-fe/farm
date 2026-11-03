"""Version selection contracts with real source graphs and explicit registry fixtures."""
# cspell:words importlib urllib
import copy
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock
import urllib.error


def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

module=load('registry_selection_tests','release-registry-selection.py')
entry=load('registry_entry_tests','incremental-release-entry.py')
fixtures=load('registry_selection_fixtures','incremental-chain-plan.test.py')


def source_fixture():
    files,family,source=fixtures.fixture()
    files['packages/core/package.json']=b'{"name":"@farmfe/core","version":"1.0.0","private":true}'
    files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
    files['docs/releases/v2-candidate.json']=json.dumps(family).encode()
    return {**source,'files':files,'modes':{name:'100644' for name in files}},family


def observed(row,published=False):
    value={'name':row['name'],'version':row['version'],'status':'published' if published else 'missing'}
    if published:value.update(archiveSha256='c'*64,integrity='sha512-'+'YQ==',metadata={})
    return value


class Contracts(unittest.TestCase):
    def test_no_baseline_missing_versions_build_full_family(self):
        source,family=source_fixture();snapshot=module.collect(source,'npm',observed)
        result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],['npm:a','npm:b']);self.assertEqual(result['versionRequired'],[])
        self.assertFalse(result['publicationBlocked']);self.assertIsNone(result['baselineSHA'])

    def test_only_missing_version_uploads_but_published_dependency_builds(self):
        source,family=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,row['name']=='a'))
        result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],['npm:b']);self.assertEqual(result['build'],['npm:b'])
        self.assertIn('npm:packages/a',result['buildNodes']);self.assertIn('npm:a',result['buildDependencyClosure'])
        self.assertIn('npm:a',result['registryRetained'])

    def test_control_and_product_diff_do_not_republish_unchanged_versions(self):
        source,family=source_fixture();source['files']['scripts/new-control.py']=b'changed';source['files']['packages/a/src/index.ts']=b'changed'
        source['modes']={name:'100644' for name in source['files']}
        snapshot=module.collect(source,'npm',lambda row:observed(row,True));result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],[]);self.assertEqual(result['buildNodes'],[]);self.assertEqual(result['versionRequired'],[])

    def test_unknown_error_missing_family_wrong_source_and_digest_reject(self):
        for defect in ['error','missing','source','checksum']:
            source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,True))
            if defect=='error':snapshot['packages']['npm:a']['status']='error'
            elif defect=='missing':del snapshot['packages']['npm:a']
            elif defect=='source':snapshot['sourceSHA']='f'*40
            else:snapshot['packages']['npm:a']['archiveSha256']='invalid'
            with self.assertRaises(ValueError):entry.source_plan(source,registry_snapshot=snapshot)

    def test_registry_404_only_is_absence(self):
        row={'kind':'npm','name':'a','version':'1.0.0'}
        for code in [404,401,403,429,500,502]:
            reader=mock.Mock();reader.json.side_effect=urllib.error.HTTPError('https://registry.npmjs.org/a/1.0.0',code,'fixture',{},None)
            if code==404:self.assertEqual(module.Registry(reader).lookup(row)['status'],'missing')
            else:
                with self.assertRaises(urllib.error.HTTPError):module.Registry(reader).lookup(row)
        for error in [TimeoutError(),ValueError('invalid JSON')]:
            reader=mock.Mock();reader.json.side_effect=error
            with self.assertRaises(type(error)):module.Registry(reader).lookup(row)

    def test_published_dependency_declarations_and_checksum_rechecked(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,True))
        snapshot['packages']['npm:b']['metadata']={'dependencies':{'a':'2.0.0'}}
        with self.assertRaisesRegex(ValueError,'dependency incompatible'):entry.source_plan(source,registry_snapshot=snapshot)
        snapshot=module.collect(source,'npm',lambda row:observed(row,True))
        with self.assertRaisesRegex(ValueError,'Retained published version'):module.verify_retained(source,snapshot,observed)

    def test_stable_range_bounds_and_unknown_tags_fail_closed(self):
        for version,requirement,result in [('2.0.0','2.0.0',True),('2.0.1','2.0.0',False),('2.3.0','^2.0.0',True),('3.0.0','^2.0.0',False),('0.2.3','^0.2.0',True),('0.3.0','^0.2.0',False),('0.0.2','^0.0.1',False),('2.1.0','~2.0.0',False)]:
            self.assertEqual(module.stable_satisfies(version,requirement),result)
        with self.assertRaises(ValueError):module.stable_satisfies('2.0.0','latest')

    def test_frozen_missing_set_survives_partial_publication(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',observed)
        before=entry.source_plan(source,registry_snapshot=snapshot)
        module.verify_retained(source,snapshot,lambda row:observed(row,True))
        self.assertEqual(entry.source_plan(source,registry_snapshot=snapshot),before)
        # Fresh plans see registry changes; an existing transaction keeps originals.
        after=entry.source_plan(source,registry_snapshot=module.collect(source,'npm',lambda row:observed(row,True)))
        self.assertEqual(after['publish'],[]);self.assertNotEqual(before['planHash'],after['planHash'])

    def test_state_baseline_fence_changes_hash_without_changing_upload_scope(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',observed)
        before=entry.source_plan(source,registry_snapshot=snapshot);snapshot['stateBaseline']='e'*64
        after=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(before['publish'],after['publish']);self.assertNotEqual(before['planHash'],after['planHash'])
        self.assertEqual(after['baselineBinding'],'e'*64)

    def test_locked_registry_revalidation_preserves_snapshot_and_rejects_drift(self):
        publisher=load('registry_locked_publisher','incremental-release-publish.py')
        source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,row['name']=='a'))
        plan=entry.source_plan(source,registry_snapshot=snapshot)
        manifest={key:plan[key] for key in ['sourceSHA','sourceTree','planHash','versionRequired','publish']}
        manifest.update(registrySnapshot=snapshot,baseline=None,rebuild=plan['build'])
        transaction=mock.Mock();transaction.manifest=manifest
        transaction.owned.return_value=(None,{'active':{'phase':'uploading','packages':{}}})
        registry=mock.Mock();registry.lookup.side_effect=lambda row:observed(row,True)
        baseline_reader=mock.Mock(side_effect=AssertionError('No historical baseline lookup permitted'))
        with mock.patch.object(publisher.entry,'load',return_value=module),mock.patch.object(module,'Registry',return_value=registry):
            result=publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            self.assertEqual(result,plan);baseline_reader.assert_not_called()
            registry.lookup.side_effect=TimeoutError('registry unavailable')
            with self.assertRaises(TimeoutError):publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            transaction.reject_unwritten.assert_called_once()
            transaction.reject_unwritten.reset_mock()
            # After any possible upload, a failure preserves the original transaction.
            transaction.owned.return_value=(None,{'active':{'phase':'uploading','packages':{'npm:b':{'status':'started'}}}})
            with self.assertRaises(TimeoutError):publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            transaction.reject_unwritten.assert_not_called()

    def test_cargo_versions_come_from_fixed_sdk_manifests_not_npm_candidate(self):
        sha=entry.chain.impact.git('rev-parse','HEAD').decode().strip();source=entry.evidence.source_reader(sha)
        records=module.records(source,'cargo')
        row=records['cargo:farmfe_core'];file=row['manifestPath']
        parts=list(map(int,row['version'].split('.')));parts[2]+=1;next_version='.'.join(map(str,parts))
        before=source['files'][file];after=before.replace(('version = "'+row['version']+'"').encode(),('version = "'+next_version+'"').encode(),1)
        self.assertNotEqual(before,after)
        source['files'][file]=after
        snapshot=module.collect(source,'cargo',observed)
        self.assertEqual(snapshot['packages']['cargo:farmfe_core']['version'],next_version)
        self.assertEqual(len(module.validate(source,'cargo',snapshot)),28)
        old=next(row for row in json.loads(source['files']['docs/releases/v2-candidate.json'])['cargo'] if row['name']=='farmfe_core')
        self.assertEqual(old['version'],row['version'])

    def test_real_fixed_main_family_and_shared_graph(self):
        sha=entry.chain.impact.git('rev-parse','HEAD').decode().strip();source=entry.evidence.source_reader(sha)
        snapshot=module.collect(source,'npm',observed);plan=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(len(plan['publish']),297);self.assertEqual(len(module.records(source,'cargo')),28)
        self.assertEqual(plan['versionRequired'],[]);self.assertTrue(plan['preparationNativeOwners'])
        self.assertIn('cargo:crates/core',plan['buildNodes'])

if __name__=='__main__':unittest.main()
