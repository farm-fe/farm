"""Completed lineage must cover the full source family and public registry bytes."""
# cspell:words importlib
import base64
import copy
import hashlib
import importlib.util
import json
import io
from pathlib import Path
import unittest
import zipfile
from unittest import mock

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

module=load('completed_release_contract','incremental-release-baseline.py')
fixtures=load('completed_release_fixture','incremental-chain-plan.test.py')
entry=load('completed_release_entry','incremental-release-entry.py')
entry_fixtures=load('completed_original_zip_fixture','incremental-release-entry.test.py')

class Contracts(unittest.TestCase):
    def setUp(self):
        files,family,identity=fixtures.fixture()
        files['packages/core/package.json']=json.dumps({'name':'@farmfe/core','version':'1.0.0','private':True}).encode()
        files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
        files['docs/releases/v2-candidate.json']=json.dumps(family).encode()
        self.source={**identity,'files':files,'modes':fixtures.modes(files),'policyHash':'pinned'}
        self.family=family
        rows=[{**row,'archiveSha256':'e'*64,'archiveBytes':1} for row in module.chain.family_records(family).values() if row['kind']=='npm']
        self.manifest={'publicationScope':'npm','chain':'release','profile':'release','namespace':'farm-release-v1','sourceSHA':identity['sha'],'sourceTree':identity['tree'],
                       'producerRunId':10,'producerAttempt':1,'baseline':None,'packages':rows,'tag':'v1.0.0','planHash':'f'*64}
        self.transaction=module.chain.digest(self.manifest)
        self.state={'baseline':self.transaction,'active':None,'completed':{self.transaction:{'manifest':self.manifest}}}
        self.registry=mock.Mock();self.registry.verify.side_effect=lambda records:{key:row['archiveSha256'] for key,row in records.items()}
        self.transport=mock.Mock();self.transport.get.return_value={'id':1,'default_branch':'main'}
        self.reader=module.ReleaseBaselineReader(self.transport,self.registry,lambda _:self.source)
        self.reader.persistent=mock.Mock(return_value=None)
        self.reader.state=mock.Mock(side_effect=lambda:('1'*40,self.state))
        self.reader.completed=mock.Mock(side_effect=lambda transaction,record,repository:(record['manifest'],self.source))
    def test_no_completed_baseline_is_a_first_build_not_an_invented_success(self):
        self.state['baseline']=None
        self.assertIsNone(self.reader.lookup());self.registry.verify.assert_not_called()
    def test_successful_lineage_produces_independent_anchor_and_non_full_no_change_plan(self):
        result=self.reader.lookup();self.registry.verify.assert_called_once()
        plan=entry.source_plan(self.source,result)
        self.assertEqual(plan['build'],[]);self.assertEqual(plan['publish'],[]);self.assertEqual(plan['versionRequired'],[])
        self.assertEqual(plan['baselineBinding'],self.transaction)
    def test_partial_family_is_rejected(self):
        self.manifest['packages'].pop()
        changed=module.chain.digest(self.manifest);self.state['baseline']=changed;self.state['completed']={changed:{'manifest':self.manifest}}
        with self.assertRaisesRegex(ValueError,'family'):self.reader.lookup()
    def test_registry_claims_cannot_verify_themselves(self):
        self.registry.verify.return_value={};self.registry.verify.side_effect=None
        with self.assertRaisesRegex(ValueError,'registry byte'):self.reader.lookup()
    def test_moved_state_or_wrong_actual_abi_is_rejected(self):
        self.reader.state.side_effect=[('1'*40,self.state),('2'*40,self.state)]
        with self.assertRaisesRegex(ValueError,'moved'):self.reader.lookup()
        self.reader.state.side_effect=lambda:('1'*40,self.state)
        self.source['files']['crates/core/src/lib.rs']+=b'\n'
        self.source['files']['crates/core/src/lib.rs']=self.source['files']['crates/core/src/lib.rs'].replace(b'"0.5.4"',b'"0.5.5"')
        with self.assertRaisesRegex(ValueError,'actual source declaration'):self.reader.lookup()
    def test_new_owner_is_a_real_publication_delta_and_removal_requires_authorization(self):
        baseline=self.reader.lookup();current=copy.deepcopy(self.source)
        current['sha']='c'*40;current['tree']='d'*40
        current['files']['packages/new/package.json']=b'{"name":"new-owner","version":"1.0.0"}'
        current['files']['packages/new/src/index.ts']=b'export const value=1'
        current['modes'].update({'packages/new/package.json':'100644','packages/new/src/index.ts':'100644'})
        family=json.loads(current['files']['docs/releases/v2-candidate.json'])
        family['npm']['primary'].append({'name':'new-owner','version':'1.0.0','manifestPath':'packages/new/package.json'})
        current['files']['docs/releases/v2-candidate.json']=json.dumps(family).encode()
        result=entry.source_plan(current,baseline)
        self.assertEqual(result['publish'],['npm:new-owner']);self.assertEqual(result['registryCollisionChecks'],['npm:new-owner'])
        for file in list(current['files']):
            if file.startswith('packages/b/'):
                current['files'].pop(file);current['modes'].pop(file)
        pkg=json.loads(current['files']['examples/one/package.json']);pkg['dependencies']={};current['files']['examples/one/package.json']=json.dumps(pkg).encode()
        family['npm']['primary']=[row for row in family['npm']['primary'] if row['name']!='b']
        current['files']['docs/releases/v2-candidate.json']=json.dumps(family).encode()
        result=entry.source_plan(current,baseline)
        self.assertEqual(result['removedPackages'],['npm:b']);self.assertTrue(result['publicationBlocked'])
        self.assertEqual(result['publicationBlockReason'],'retirement-authorization-required')
    def test_source_change_without_version_is_blocked_then_exact_version_delta_is_publishable(self):
        baseline=self.reader.lookup();current=copy.deepcopy(self.source);current['sha']='c'*40;current['tree']='d'*40
        current['files']['packages/a/src/index.ts']+=b' changed'
        plan=entry.source_plan(current,baseline)
        self.assertEqual(plan['versionRequired'],['npm:a','npm:b'])
        for directory in ['packages/a','packages/b']:
            file=directory+'/package.json';pkg=json.loads(current['files'][file]);pkg['version']='1.0.1';current['files'][file]=json.dumps(pkg).encode()
        candidate=json.loads(current['files']['docs/releases/v2-candidate.json'])
        for row in candidate['npm']['primary']:row['version']='1.0.1'
        current['files']['docs/releases/v2-candidate.json']=json.dumps(candidate).encode()
        plan=entry.source_plan(current,baseline)
        self.assertEqual(plan['publish'],['npm:a','npm:b']);self.assertEqual(plan['versionRequired'],[])
    def test_registry_transport_url_is_not_plan_identity_but_archive_bytes_are(self):
        baseline=self.reader.lookup();baseline['ledger']['packages']['npm:a']['tarball']='https://registry.npmjs.org/a/-/old.tgz'
        baseline['anchor']['ledgerSha256']=module.chain.digest(baseline['ledger'])
        first=entry.source_plan(self.source,baseline)
        baseline['ledger']['packages']['npm:a']['tarball']='https://registry.npmjs.org/a/-/new-location.tgz'
        baseline['anchor']['ledgerSha256']=module.chain.digest(baseline['ledger'])
        self.assertEqual(entry.source_plan(self.source,baseline),first)
        baseline['ledger']['packages']['npm:a']['archiveSha256']='f'*64
        baseline['anchor']['ledgerSha256']=module.chain.digest(baseline['ledger']);baseline['anchor']['registryChecksums']['npm:a']='f'*64
        self.assertNotEqual(entry.source_plan(self.source,baseline)['planHash'],first['planHash'])
    def test_state_reader_verifies_actual_git_blob_and_fixed_path(self):
        body=json.dumps(self.state).encode();oid=hashlib.sha1(b'blob '+str(len(body)).encode()+b'\0'+body).hexdigest()
        responses=[{'ref':'refs/heads/farm-release-state','object':{'type':'commit','sha':'1'*40}}],{'tree':{'sha':'2'*40}},
        tree={'truncated':False,'tree':[{'path':'publication-state.json','mode':'100644','type':'blob','sha':oid}]}
        blob={'encoding':'base64','size':len(body),'content':base64.b64encode(body).decode()}
        reader=module.ReleaseBaselineReader(self.transport,self.registry)
        self.transport.get.side_effect=[responses[0],responses[1],tree,blob]
        self.assertEqual(reader.state()[1],self.state)
        blob['content']=base64.b64encode(body+b' ').decode();self.transport.get.side_effect=[responses[0],responses[1],tree,blob]
        with self.assertRaises(ValueError):reader.state()
    def test_completed_provenance_reads_real_signed_seal_public_and_all_platform_receipts(self):
        publisher=load('baseline_full_release_fixture','incremental-release-publish.test.py')
        test=publisher.Contracts()
        fixture,_,original,current=test.recovery_fixture(1,3,{'ubuntu-24.04':2,'macos-latest':1,'windows-latest':3})
        manifest=fixture.manifest;transaction=module.chain.digest(manifest)
        source={'sha':fixture.source['sha'],'tree':fixture.source['tree']}
        # Freeze the external-verifier fixture result BEFORE mutating API/content.
        # Nothing below derives a trusted digest from a mutated receipt.
        saved={scope:copy.deepcopy(fixture.proof_provider(fixture.transport,run,source,scope))
               for scope,run in [('frozen',original),('public',current)]}
        def verified(client,run,identity,scope):
            expected,rows=copy.deepcopy(saved[scope])
            module.check(run['run_attempt']==expected['attempt'] and {key:identity.get(key) for key in ['sha','tree']}==source,'Fixture signed source/attempt differs')
            return expected,rows
        old=fixture.transport.get.side_effect
        def get(route):
            if route=='/git/ref/tags/v1.0.0':return {'object':{'type':'commit','sha':source['sha']}}
            if route=='/releases/tags/v1.0.0':return {'draft':False,'prerelease':False,'body':'<!-- farm-release-manifest:'+transaction+' -->'}
            return old(route)
        fixture.transport.get.side_effect=get
        reader=module.ReleaseBaselineReader(fixture.transport,self.registry,lambda _:fixture.source,proof_provider=verified)
        self.assertEqual(reader.completed(transaction,{'manifest':manifest},fixture.repository)[0],manifest)
        for defect in ['candidate-bytes','public-bytes','seal-digest','latest-failure','wrong-source','missing-upload']:
            metadata=copy.deepcopy(fixture.candidate_metadata);bodies=dict(fixture.candidate_bodies)
            jobs=copy.deepcopy(fixture.jobs_by_attempt);run=copy.deepcopy(current)
            if defect=='candidate-bytes':fixture.candidate_bodies[200]+=b'tampered'
            if defect=='public-bytes':fixture.candidate_bodies[400]+=b'tampered'
            if defect=='seal-digest':fixture.candidate_metadata[61]['digest']='sha256:'+'0'*64
            if defect=='latest-failure':
                next(row for row in fixture.jobs_by_attempt[3] if row['name']=='Independent release candidate consumer (macos-latest)')['conclusion']='failure'
            if defect=='wrong-source':current['head_sha']='f'*40
            if defect=='missing-upload':
                next(row for row in fixture.jobs_by_attempt[3] if row['name']==entry.CONSUMER)['steps']=[]
            with self.subTest(defect=defect),self.assertRaises(ValueError):reader.completed(transaction,{'manifest':manifest},fixture.repository)
            fixture.candidate_metadata.clear();fixture.candidate_metadata.update(metadata)
            fixture.candidate_bodies.clear();fixture.candidate_bodies.update(bodies)
            fixture.jobs_by_attempt.clear();fixture.jobs_by_attempt.update(jobs)
            current.clear();current.update(run)

if __name__=='__main__':unittest.main(verbosity=2)
