"""Release retry policy with explicit offline cryptographic transport fixtures."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

m=load('lifecycle_provenance','release-provenance.py')
fixtures=load('lifecycle_fixtures','release-provenance.test.py')

class Contracts(unittest.TestCase):
    def test_finalize_retry_requires_new_public_signer_not_carried_old_public_success(self):
        api,old,source,state=fixtures.EnvelopeContracts().fixture()
        state['latest'].update(status='in_progress',conclusion=None)
        def verified(subject,bundle,expected):
            self.assertEqual(expected['attempt'],2)
            return {cell:{'offlineOnly':True} for cell in expected['cells']}
        with self.assertRaisesRegex(ValueError,'current authoritative attempt'):
            m.verified_run(api,old,source,'public',verify=verified,signer_sha='c'*40)
        # Re-executing the public signer and its dependent finalize job creates the
        # new attempt's actual envelope. This fixture models that transport only.
        current=copy.deepcopy(state['latest'])
        state['artifact']['name']='farm-release-bindings-public-42-2'
        expected,rows=m.verified_run(api,current,source,'public',verify=verified,signer_sha='c'*40)
        self.assertEqual(expected['attempt'],2)
        self.assertEqual(len(rows),7)
        self.assertNotIn('frozen',rows)

if __name__=='__main__':unittest.main(verbosity=2)
