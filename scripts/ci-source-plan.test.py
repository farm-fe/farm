"""Real committed repository and historical global-coverage regression."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('tested_source_plan', Path(__file__).with_name('ci-source-plan.py'))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
BASE = '1d1039e7cbdf9858432434734b3c06da5c8a7779'


class Contracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.head = module.impact.git('rev-parse', 'HEAD').decode().strip()
        cls.files, cls.hashes = module.impact.snapshot(cls.head)
        cls.old_files, cls.old_hashes = module.impact.snapshot(BASE)
        cls.plan = module.make_plan(BASE, cls.head, cls.head)

    def test_real_old_main_four_snapshots_and_independent_functional_recompute(self):
        plan = self.plan
        self.assertNotIn('scripts/directory-impact-rules.json', self.old_files)
        self.assertTrue(plan['fullRequired'])
        self.assertFalse(plan['skipAuthorized']); self.assertFalse(plan['reuseAuthorized'])
        self.assertEqual(plan['sourceGraphs']['base']['policy'], module.HISTORICAL)
        self.assertEqual(plan['sourceGraphs']['tested']['rulesSource'], 'committed-file')
        self.assertLess(len(json.dumps(plan).encode()), 4 * 1024 * 1024)
        module.verify_main_plan(plan, 'e2e')
        for group in [['electron', 'public-path', 'react'], ['sass-plugin-rust', 'sass'], ['vite-adapter-vue2.7', 'vite-adapter-vue2']]:
            self.assertTrue(all('npm:examples/' + name in plan['graph'] for name in group))

    def test_tested_merge_keeps_its_own_source_and_cannot_impersonate_main(self):
        plan = module.make_plan(BASE, BASE, self.head)
        self.assertEqual(plan['sourceGraphs']['head']['policy'], module.HISTORICAL)
        self.assertEqual(plan['sourceGraphs']['tested']['rulesSource'], 'committed-file')
        self.assertNotEqual(plan['sourceGraphs']['head']['sourceRecordsDigest'], plan['sourceGraphs']['tested']['sourceRecordsDigest'])
        with self.assertRaisesRegex(ValueError, 'exact head commit'):
            module.verify_main_plan(plan, 'e2e')

    def test_real_historical_global_node_connects_every_node_and_every_fingerprint(self):
        graph, metadata = module.committed_graph(self.old_files, self.old_hashes, historical_allowed=True)
        key = 'input:historical-all-committed'
        for owner, node in graph.items():
            if owner != key:
                for field in ['productDeps', 'preparationDeps', 'testDeps']:
                    self.assertIn(key, node[field])
        before = module.fingerprints(graph, metadata)
        files = dict(self.old_files); hashes = copy.deepcopy(self.old_hashes)
        files['unclassified/input.bin'] = bytes([0, 255]) + b'unknown'
        hashes['unclassified/input.bin'] = {'mode': '100644', 'sha256': hashlib.sha256(files['unclassified/input.bin']).hexdigest()}
        after_graph, after_meta = module.committed_graph(files, hashes, historical_allowed=True)
        after = module.fingerprints(after_graph, after_meta)
        self.assertTrue(all(before[owner]['product'] != after[owner]['product'] and before[owner]['tests'] != after[owner]['tests'] for owner in before))

    def test_historical_delete_rename_mode_and_byte_changes_all_change_binding(self):
        for mutation in ['delete', 'rename', 'mode', 'bytes']:
            files = dict(self.old_files); hashes = copy.deepcopy(self.old_hashes)
            # Use a non-manifest input so graph parsing is independently valid.
            name = 'docs/history-source-proof.txt'; files[name] = b'first'
            hashes[name] = {'mode': '100644', 'sha256': hashlib.sha256(files[name]).hexdigest()}
            _, prior = module.committed_graph(files, hashes, historical_allowed=True)
            if mutation == 'delete': del files[name]; del hashes[name]
            elif mutation == 'rename': files[name + '.renamed'] = files.pop(name); hashes[name + '.renamed'] = hashes.pop(name)
            elif mutation == 'mode': hashes[name]['mode'] = '100755'
            else: files[name] = b'second'; hashes[name]['sha256'] = hashlib.sha256(files[name]).hexdigest()
            _, changed = module.committed_graph(files, hashes, historical_allowed=True)
            self.assertNotEqual(prior['sourceRecordsDigest'], changed['sourceRecordsDigest'])

    def test_tampered_plan_cannot_self_certify_by_rehashing(self):
        for mutation in ['policy', 'records', 'count', 'edge', 'full', 'skip', 'reuse', 'complete', 'scope']:
            plan = copy.deepcopy(self.plan)
            if mutation == 'policy': plan['sourceGraphs']['base']['policy'] = 'manifest-only'
            elif mutation == 'records': plan['sourceGraphs']['base']['sourceRecordsDigest'] = '0' * 64
            elif mutation == 'count': plan['sourceGraphs']['base']['sourceFileCount'] -= 1
            elif mutation == 'edge': plan['graph']['npm:packages/core']['productDeps'] = []
            elif mutation == 'full': plan['fullRequired'] = False
            elif mutation == 'skip': plan['skipAuthorized'] = True
            elif mutation == 'reuse': plan['reuseAuthorized'] = True
            elif mutation == 'complete': plan['sourceInputProofComplete'] = False
            else: plan['sourceInputProofScope'] = 'complete-environment'
            plan['planHash'] = module.impact.digest({key: value for key, value in plan.items() if key != 'planHash'})
            with self.assertRaises(ValueError, msg=mutation): module.verify_main_plan(plan, 'e2e')

    def test_current_rules_global_inputs_do_not_hide_behind_package_owners(self):
        for file in ['examples/ssr-utils/helper.ts', 'e2e/fixtures/input.ts']:
            with mock.patch.object(module.impact, 'changed_paths', return_value=[{'status': 'M', 'paths': [file]}]):
                plan = module.make_plan(self.head, self.head, self.head)
            self.assertTrue(plan['fullRequired'])
            self.assertTrue(any(row['reason'] == 'declared-global-input' for row in plan['reasons']))
        with mock.patch.object(module.impact, 'changed_paths', return_value=[{'status': 'M', 'paths': ['packages/utils/src/index.ts']}]):
            plan = module.make_plan(self.head, self.head, self.head)
        self.assertIn('input:validation:packages/utils/', plan['testNodes'])
        self.assertIn('npm:examples/react', plan['testNodes'])

    def test_missing_tested_rules_and_missing_or_forged_records_reject(self):
        with self.assertRaises(ValueError): module.committed_graph(self.old_files, self.old_hashes, historical_allowed=False)
        hashes = copy.deepcopy(self.hashes); hashes.pop(next(iter(hashes)))
        with self.assertRaises(ValueError): module.committed_graph(self.files, hashes, historical_allowed=False)
        hashes = copy.deepcopy(self.hashes); hashes['package.json']['sha256'] = '0' * 64
        with self.assertRaises(ValueError): module.committed_graph(self.files, hashes, historical_allowed=False)

    def test_actual_publication_and_dependency_ambiguity_still_reject(self):
        for path, body in [('packages/duplicate-core/package.json', self.files['packages/core/package.json']),
                           ('examples/ambiguous-client/package.json', json.dumps({'name': 'private-client', 'private': True, 'dependencies': {'@farmfe-examples/react': 'workspace:*'}}).encode())]:
            files = dict(self.files); hashes = copy.deepcopy(self.hashes)
            files[path] = body; hashes[path] = {'mode': '100644', 'sha256': hashlib.sha256(body).hexdigest()}
            with self.assertRaises(ValueError): module.committed_graph(files, hashes, historical_allowed=False)

    def test_actual_old_main_plan_through_non_publishing_functional_api_replay(self):
        spec = importlib.util.spec_from_file_location('real_functional_fixture', Path(__file__).with_name('incremental-release-functional.test.py'))
        fixture = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixture)
        case = fixture.Contracts()
        case.sha = self.head; case.tree = self.plan['testedTree']
        case.plans = {'e2e': self.plan, 'unit': module.make_plan(BASE, self.head, self.head, 'unit')}
        result = case.verify(case.fixture())
        self.assertTrue(result['functionalAcceptanceVerified']); self.assertFalse(result['buildBytesReused'])


if __name__ == '__main__':
    unittest.main(verbosity=2)
