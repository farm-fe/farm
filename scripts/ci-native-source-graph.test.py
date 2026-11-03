"""Fixed tracked snapshot regression for directory identities in native ledgers."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

spec=importlib.util.spec_from_file_location('native_source_evidence',Path(__file__).with_name('ci-native-evidence.py'))
evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)


class Contracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.sha=evidence.impact.git('rev-parse','HEAD').decode().strip()
        cls.files,cls.hashes=evidence.impact.snapshot(cls.sha)

    def changed(self,path,body):
        files=dict(self.files);hashes=copy.deepcopy(self.hashes)
        files[path]=body;hashes[path]={'mode':'100644','sha256':hashlib.sha256(body).hexdigest()}
        return files,hashes

    def test_three_real_duplicate_example_groups_no_longer_break_native_owners(self):
        groups=[['electron','public-path','react'],['sass-plugin-rust','sass'],['vite-adapter-vue2.7','vite-adapter-vue2']]
        for group in groups:
            names={json.loads(self.files['examples/'+name+'/package.json'])['name'] for name in group}
            self.assertEqual(len(names),1)
        with self.assertRaises(evidence.impact.WorkspaceIdentityError):evidence.impact.graph(self.files)
        result=evidence.source_inputs(self.files,self.hashes,['npm:@farmfe/core','npm:@farmfe/plugin-react'])
        self.assertEqual(set(result),{'npm:@farmfe/core','npm:@farmfe/plugin-react'})
        self.assertTrue(all(len(value)==64 for value in result.values()))

    def test_ambiguous_requested_owner_and_actual_dependency_still_reject(self):
        with self.assertRaisesRegex(ValueError,'owner is missing or ambiguous'):
            evidence.source_inputs(self.files,self.hashes,['npm:@farmfe-examples/react'])
        path='rust-plugins/react/package.json';value=json.loads(self.files[path])
        value.setdefault('dependencies',{})['@farmfe-examples/react']='workspace:*'
        files,hashes=self.changed(path,json.dumps(value).encode())
        with self.assertRaisesRegex(ValueError,'ambiguous-dependency'):
            evidence.source_inputs(files,hashes,['npm:@farmfe/plugin-react'])

    def test_publication_name_collision_rejects_even_when_not_requested(self):
        path='js-plugins/duplicate-source-owner/package.json'
        files,hashes=self.changed(path,json.dumps({'name':'@farmfe/core','version':'2.0.0'}).encode())
        with self.assertRaises(ValueError):evidence.source_inputs(files,hashes,['npm:@farmfe/plugin-react'])

    def test_all_publishable_cargo_workspace_roots_reject_duplicate_identity(self):
        files=dict(self.files);hashes=copy.deepcopy(self.hashes)
        for directory in ['rust-ecosystem/ledger-one','rust-ecosystem/ledger-two']:
            path=directory+'/Cargo.toml';body=b'[package]\nname = "native_ledger_duplicate"\nversion = "1.0.0"\n'
            files[path]=body;hashes[path]={'mode':'100644','sha256':hashlib.sha256(body).hexdigest()}
        with self.assertRaisesRegex(ValueError,'Ambiguous publication package identity'):
            evidence.source_inputs(files,hashes,['npm:@farmfe/core'])

    def test_cargo_tools_and_global_inputs_remain_fingerprint_inputs(self):
        owners=['npm:@farmfe/core','npm:@farmfe/plugin-react']
        original=evidence.source_inputs(self.files,self.hashes,owners)
        for path in ['crates/core/src/lib.rs','packages/plugin-tools/src/ledger-probe.ts','scripts/ledger-probe.py','Cargo.lock']:
            files,hashes=self.changed(path,self.files.get(path,b'')+b'\n# source probe\n')
            actual=evidence.source_inputs(files,hashes,owners)
            self.assertNotEqual(actual['npm:@farmfe/plugin-react'],original['npm:@farmfe/plugin-react'],path)
            if path!='packages/plugin-tools/src/ledger-probe.ts':self.assertNotEqual(actual['npm:@farmfe/core'],original['npm:@farmfe/core'],path)

    def test_rules_are_taken_from_fixed_snapshot_and_directory_edges_are_bound(self):
        owners=['npm:@farmfe/plugin-react'];original=evidence.source_inputs(self.files,self.hashes,owners)
        rules=json.loads(self.files['scripts/directory-impact-rules.json'])
        rules['dependencies'].append({'consumer':'rust-plugins/react','dependencies':['examples/electron'],'edges':['product']})
        files,hashes=self.changed('scripts/directory-impact-rules.json',json.dumps(rules).encode())
        self.assertNotEqual(evidence.source_inputs(files,hashes,owners),original)
        # Path-selected local dependency is unambiguous despite the shared name.
        value=json.loads(files['rust-plugins/react/package.json']);value.setdefault('dependencies',{})['example-alias']='file:../../examples/electron'
        files['rust-plugins/react/package.json']=json.dumps(value).encode()
        hashes['rust-plugins/react/package.json']={'mode':'100644','sha256':hashlib.sha256(files['rust-plugins/react/package.json']).hexdigest()}
        self.assertEqual(set(evidence.source_inputs(files,hashes,owners)),set(owners))

    def test_actual_capture_uses_committed_tree_and_retains_no_authority(self):
        with tempfile.TemporaryDirectory() as tmp:
            products=Path(tmp)/'products.json';output=Path(tmp)/'source.json'
            products.write_text(json.dumps({'records':[{'owner':'@farmfe/core'},{'owner':'@farmfe/plugin-react'}],'abi':'linux-arm64-gnu','profile':'ci'}))
            with mock.patch.dict(os.environ,{'GITHUB_SHA':self.sha,'GITHUB_RUN_ID':'42','GITHUB_RUN_ATTEMPT':'2','GITHUB_JOB':'build'}):evidence.capture_source(products,output)
            row=json.loads(output.read_text())
            self.assertEqual(row['source']['commitSHA'],self.sha)
            self.assertEqual(row['source']['tree'],evidence.impact.git('rev-parse',self.sha+'^{tree}').decode().strip())
            self.assertEqual((row['abi'],row['profile']),('linux-arm64-gnu','ci'))
            self.assertFalse(row['reuseAuthorized']);self.assertFalse(row['acceptanceInherited'])
            self.assertIn('scripts/incremental-chain-plan.py',row['policyFiles'])
            self.assertIn('scripts/directory-impact-rules.json',row['policyFiles'])

if __name__=='__main__':unittest.main()
