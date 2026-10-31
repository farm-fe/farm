"""Persistent repository receipts survive Actions expiry without claiming attestation."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest
from unittest import mock

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

module=load('checkpoint_contract','incremental-release-checkpoint.py')
baseline=load('checkpoint_reader','incremental-release-baseline.py')
fixtures=load('checkpoint_fixture','incremental-release-baseline.test.py')

class Contracts(unittest.TestCase):
    def setUp(self):
        self.fixture=fixtures.Contracts();self.fixture.setUp()
        self.source,self.manifest,self.transaction=self.fixture.source,self.fixture.manifest,self.fixture.transaction
        proof={'verified':True,'manifestHash':self.transaction,'sourceSHA':self.source['sha'],'sourceTree':self.source['tree']}
        self.value=module.create(self.manifest,self.source,None,proof,1)
        self.release={'draft':False,'prerelease':False,'author':{'login':'github-actions[bot]','type':'Bot'},
                      'body':'<!-- farm-release-manifest:'+self.transaction+' -->\n'+module.encode(self.value)}
    def test_complete_receipt_roundtrip_binds_source_family_and_origin(self):
        self.assertEqual(module.decode(self.release,self.manifest,self.source,1),self.value)
        self.assertEqual(self.value['kind'],'repository-release-checkpoint-not-actions-attestation')
    def test_wrong_author_profile_source_partial_family_or_bad_bytes_are_rejected(self):
        for mutation in ['author','profile','source','family','bytes']:
            release=copy.deepcopy(self.release);value=copy.deepcopy(self.value)
            if mutation=='author':release['author']['login']='other'
            elif mutation=='profile':value['profile']='ci'
            elif mutation=='source':value['sourceTree']='f'*40
            elif mutation=='family':value['packages'].pop(next(iter(value['packages'])))
            else:release['body']=release['body'].replace(self.transaction,'0'*64)
            if mutation not in ['author','bytes']:release['body']='<!-- farm-release-manifest:'+self.transaction+' -->\n'+module.encode(value)
            with self.assertRaises(ValueError):module.decode(release,self.manifest,self.source,1)
    def test_verified_durable_receipt_collapses_old_lineage_without_actions_or_log_access(self):
        calls=[]
        def get(route):
            calls.append(route)
            if route=='':return {'id':1,'default_branch':'main'}
            if route=='/branches/main':return {'commit':{'sha':self.source['sha']}}
            if route=='/releases/tags/v1.0.0':return self.release
            if route=='/git/ref/tags/v1.0.0':return {'object':{'type':'commit','sha':self.source['sha']}}
            raise AssertionError('No Actions fallback should be used: '+route)
        transport=mock.Mock();transport.get.side_effect=get
        reader=baseline.ReleaseBaselineReader(transport,self.fixture.registry,lambda _:self.source)
        # Only the complete latest checkpoint is needed, even if old Actions
        # artifacts and previous state entries are no longer available.
        reader.state=mock.Mock(return_value=('1'*40,self.fixture.state))
        result=reader.lookup()
        self.assertEqual(result['transaction'],self.transaction)
        self.assertEqual(set(result['ledger']['packages']),set(self.value['packages']))
        self.fixture.registry.verify.assert_called_once()
        self.assertFalse(any('/actions/' in route for route in calls))
    def test_new_registry_bytes_cannot_be_hidden_by_a_durable_receipt(self):
        self.fixture.registry.verify.side_effect=lambda records:{key:'0'*64 for key in records}
        reader=baseline.ReleaseBaselineReader(mock.Mock(),self.fixture.registry,lambda _:self.source)
        reader.state=mock.Mock(return_value=('1'*40,self.fixture.state));reader.persistent=mock.Mock(return_value=(self.value,self.source))
        reader.transport.get.return_value={'id':1,'default_branch':'main'}
        with self.assertRaisesRegex(ValueError,'registry byte'):reader.lookup()

if __name__=='__main__':unittest.main(verbosity=2)
