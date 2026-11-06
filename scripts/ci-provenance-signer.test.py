"""Offline signer context and production wiring contracts; no OIDC requests."""
# cspell:words importlib
import importlib.util
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('signer', Path(__file__).with_name('ci-provenance-signer.py'))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)


def environment(scope='ci'):
    return {'GITHUB_ACTIONS':'true','GITHUB_EVENT_NAME':'push','GITHUB_REF':'refs/heads/main',
            'GITHUB_REPOSITORY':m.REPOSITORY,'GITHUB_REPOSITORY_ID':str(m.REPOSITORY_ID),
            'GITHUB_JOB':'sign','RUNNER_ENVIRONMENT':'github-hosted','GITHUB_SHA':'a'*40,
            'GITHUB_RUN_ID':'42','GITHUB_RUN_ATTEMPT':'3','SIGNER_WORKFLOW_SHA':'b'*40,
            'SIGNER_WORKFLOW_REF':m.REPOSITORY+'/'+m.SIGNER+'@'+'b'*40,
            'GITHUB_WORKFLOW_REF':m.REPOSITORY+'/'+m.CALLERS[scope]+'@refs/heads/main'}


class Contracts(unittest.TestCase):
    def test_both_scopes_have_exact_fixed_product_inventory(self):
        for scope, count in [('ci',8),('unit',1)]:
            value=m.identity(environment(scope),scope)
            self.assertEqual(len(value['cells']),count)
            self.assertEqual(value['attempt'],3)
            self.assertTrue(all('shadow' not in row['artifactName'] for row in value['cells'].values()))
        self.assertIn('execution-baseline',m.cells('ci'))
        self.assertEqual(set(m.cells('ci')),{'planner','execution-baseline',*[label+':'+abi for label in ['ts','browser'] for abi in ['linux-x64-gnu','darwin-arm64','win32-x64-msvc']]})

    def test_each_context_field_missing_or_wrong_rejects_before_transport(self):
        class NoAPI:
            def get(self,route):raise AssertionError('Unauthorized context reached network boundary')
        for key in environment():
            for value in [None,'wrong']:
                env=environment()
                if value is None:env.pop(key)
                else:env[key]=value
                with self.subTest(key=key,value=value),self.assertRaises(ValueError):m.prepare(NoAPI(),env,'ci',{}, {})
        with self.assertRaises(ValueError):m.identity(environment(),'release')
        with self.assertRaises(ValueError):m.identity({**environment(),'GITHUB_RUN_ATTEMPT':'33'},'ci')

    def test_isolated_workflow_permissions_and_checkout_are_narrow(self):
        workflow=(Path(__file__).parent.parent/'.github/workflows/ci-provenance-signer.yaml').read_text()
        self.assertIn('permissions: {}',workflow)
        for permission in ['id-token: write','attestations: write','contents: read','actions: read']:
            self.assertEqual(workflow.count(permission),1)
        for denied in ['actions: write','packages: write','artifact-metadata: write','secrets: inherit','npm publish','cargo publish','actions/cache','pnpm install','workflow_dispatch:']:
            self.assertNotIn(denied,workflow)
        self.assertIn('ref: ${{ job.workflow_sha }}',workflow)
        self.assertIn('persist-credentials: false',workflow)
        self.assertIn('push-to-registry: false',workflow)
        self.assertIn('create-storage-record: false',workflow)
        self.assertIn("github.event_name == 'push' && github.ref == 'refs/heads/main'",workflow)
        self.assertNotIn('ref: ${{ github.sha }}',workflow)


if __name__=='__main__':unittest.main()
