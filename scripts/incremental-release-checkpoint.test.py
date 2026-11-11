"""Persistent repository receipts survive Actions expiry without claiming attestation."""
# cspell:words importlib reencoded finalizer
import copy
import base64
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
    def test_reencoded_receipt_cannot_forge_published_bytes_or_original_producer(self):
        for key, replacement in [('archiveSha256', 'f' * 64), ('originRunId', 999),
                                 ('originAttempt', 999), ('originSourceSHA', 'e' * 40),
                                 ('archiveBytes', 999), ('archive', 'other.tgz')]:
            with self.subTest(field=key):
                value = copy.deepcopy(self.value)
                identity = self.manifest['packages'][0]['kind'] + ':' + self.manifest['packages'][0]['name']
                value['packages'][identity][key] = replacement
                release = {**self.release, 'body': '<!-- farm-release-manifest:' + self.transaction + ' -->\n' + module.encode(value)}
                with self.assertRaisesRegex(ValueError, 'Published checkpoint row'):
                    module.decode(release, self.manifest, self.source, 1)
    def test_registry_retained_family_has_no_invented_build_origin(self):
        registry=load('checkpoint_registry_tests','release-registry-selection.test.py')
        selector=registry.module
        manifest=copy.deepcopy(self.manifest)
        def observed(row):
            value={'name':row['name'],'version':row['version'],'status':'published' if row['name']=='a' else 'missing'}
            if value['status']=='published':value.update(archiveSha256='c'*64,integrity='sha512-YQ==',metadata={})
            return value
        snapshot=selector.collect(self.source,'npm',observed,schema_version=1)
        manifest['registrySnapshot']=snapshot
        manifest['packages']=[row for row in manifest['packages'] if row['name']!='a']
        transaction=module.chain.digest(manifest)
        proof={'verified':True,'manifestHash':transaction,'sourceSHA':self.source['sha'],'sourceTree':self.source['tree']}
        value=module.create(manifest,self.source,None,proof,1)
        retained=value['packages']['npm:a']
        self.assertEqual(retained['originKind'],'independently-verified-registry')
        self.assertNotIn('originSourceSHA',retained);self.assertNotIn('originRunId',retained)
        release={**self.release,'body':'<!-- farm-release-manifest:'+transaction+' -->\n'+module.encode(value)}
        self.assertEqual(module.decode(release,manifest,self.source,1),value)
        retained['archiveSha256']='f'*64
        release['body']='<!-- farm-release-manifest:'+transaction+' -->\n'+module.encode(value)
        with self.assertRaisesRegex(ValueError,'Registry retained checkpoint'):module.decode(release,manifest,self.source,1)

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


class PresenceContracts(unittest.TestCase):
    integrity_a='sha512-'+base64.b64encode(b'a'*64).decode()
    integrity_b='sha512-'+base64.b64encode(b'b'*64).decode()
    def setUp(self):
        fixture=fixtures.Contracts();fixture.setUp()
        self.source=fixture.source
        self.original=copy.deepcopy(fixture.manifest)
        self.selector=module.load_registry_selection()
        self.configure({'a':'registry','b':'source'})

    def configure(self,origins):
        def observation(row):
            value={'name':row['name'],'version':row['version'],
                   'status':'published' if origins[row['name']]=='registry' else 'missing'}
            if value['status']=='published':
                value.update(evidenceKind='registry-metadata',integrity=self.integrity_a,metadata={},
                             tarballURL='https://registry.npmjs.org/'+row['name']+'/-/registry.tgz')
            return value
        self.manifest=copy.deepcopy(self.original)
        self.manifest.update(selectionPolicy='registry-exact-version-v2',releaseValidation='archive-integrity-v1',
                             registrySnapshot=self.selector.collect(self.source,'npm',observation),buildFailures={})
        self.manifest['packages']=[{**row,'archive':row['name']+'-candidate.tgz'} for row in self.manifest['packages']
                                   if origins[row['name']]!='registry']
        sealed={'npm:'+row['name']:row for row in self.manifest['packages']}
        dispositions={}
        for name,origin in origins.items():
            identity='npm:'+name
            if origin=='registry':
                value=copy.deepcopy(self.manifest['registrySnapshot']['packages'][identity])
            else:
                candidate=sealed[identity]
                value={'name':name,'version':candidate['version'],
                       'integrity':self.integrity_b,'metadata':{},
                       'tarballURL':'https://registry.npmjs.org/'+name+'/-/actual-public.tgz'}
                if origin=='source':
                    value.update(archiveSha256=candidate['archiveSha256'],archiveBytes=candidate['archiveBytes'])
            dispositions[identity]={**value,'originKind':origin,'latestVerified':True,
                                    'uploadAttempted':origin=='source',
                                    'evidenceKind':'registry-archive' if origin=='source' else 'registry-metadata',
                                    'registryBytesVerified':origin=='source'}
        self.proof={'verified':True,'kind':'release-version-presence-v1',
                    'manifestHash':module.chain.digest(self.manifest),
                    'sourceSHA':self.source['sha'],'sourceTree':self.source['tree'],'dispositions':dispositions,
                    'packages':{key:row.get('archiveSha256') for key,row in dispositions.items()}}
        self.value=module.create(self.manifest,self.source,None,self.proof,1)
        self.release=self.release_for(self.value)

    def release_for(self,value):
        return {'id':5,'draft':False,'prerelease':False,'author':{'login':'github-actions[bot]','type':'Bot'},
                'body':'<!-- farm-release-manifest:'+module.chain.digest(self.manifest)+' -->\n'+module.encode(value)}

    def decode(self,release=None,proof=None):
        return module.decode(self.release if release is None else release,self.manifest,self.source,1,
                             proof=self.proof if proof is None else proof)

    def test_mixed_receipt_binds_source_seal_and_registry_proof_with_truthful_origins(self):
        self.assertEqual(self.decode(),self.value)
        source=self.value['packages']['npm:b'];retained=self.value['packages']['npm:a']
        for field,value in self.manifest['packages'][0].items():self.assertEqual(source[field],value)
        self.assertEqual(source['originKind'],'source')
        self.assertEqual(source['originSourceSHA'],self.source['sha'])
        self.assertEqual(source['originSourceTree'],self.source['tree'])
        self.assertEqual(source['originRunId'],self.manifest['producerRunId'])
        self.assertEqual(source['originAttempt'],self.manifest['producerAttempt'])
        self.assertEqual(retained['originKind'],'registry')
        self.assertFalse(retained['registryBytesVerified'])
        self.assertEqual(retained['evidenceKind'],'registry-metadata')
        self.assertIsNone(self.proof['packages']['npm:a'])
        for field in ['archive','archiveSha256','archiveBytes','originSourceSHA','originSourceTree','originRunId','originAttempt']:
            self.assertNotIn(field,retained)
        self.assertEqual(self.value['publicAcceptanceHash'],module.chain.digest(self.proof))

    def test_reencoded_v2_row_or_origin_cannot_self_authenticate(self):
        changes={'archiveSha256':'0'*64,'archiveBytes':999,'integrity':'sha512-ZQ==',
                 'metadata':{'dependencies':{'other':'1.0.0'}},
                 'tarballURL':'https://registry.npmjs.org/other/-/other.tgz',
                 'originKind':'source','originRunId':999,'originSourceSHA':'e'*40,
                 'originAttempt':999,'latestVerified':False,'uploadAttempted':True}
        for identity in ['npm:a','npm:b']:
            for field,replacement in changes.items():
                with self.subTest(identity=identity,field=field):
                    value=copy.deepcopy(self.value)
                    if value['packages'][identity].get(field)==replacement:
                        replacement='registry-race' if field=='originKind' else False
                    value['packages'][identity][field]=replacement
                    with self.assertRaisesRegex(ValueError,'verified public proof'):
                        self.decode(self.release_for(value))

    def test_v2_without_verified_proof_is_rejected_even_when_body_checksum_is_valid(self):
        with self.assertRaisesRegex(ValueError,'verified public proof'):
            module.decode(self.release,self.manifest,self.source,1)
        for field,value in [('verified',False),('kind','release-byte-integrity-v1'),
                            ('manifestHash','f'*64),('sourceSHA','e'*40),('sourceTree','d'*40)]:
            proof={**self.proof,field:value}
            with self.subTest(field=field),self.assertRaises(ValueError):self.decode(proof=proof)

    def test_public_acceptance_hash_and_every_proof_field_are_bound(self):
        value=copy.deepcopy(self.value);value['publicAcceptanceHash']='0'*64
        with self.assertRaisesRegex(ValueError,'verified public proof'):
            self.decode(self.release_for(value))
        proof={**self.proof,'laterAttempt':True}
        with self.assertRaisesRegex(ValueError,'verified public proof'):self.decode(proof=proof)
        proof=copy.deepcopy(self.proof);proof['packages']['npm:b']='f'*64
        with self.assertRaisesRegex(ValueError,'checksum coverage'):self.decode(proof=proof)
        # Receipt objects cannot silently change when callers later mutate proof metadata.
        self.proof['dispositions']['npm:b']['metadata']['dependencies']={'other':'1.0.0'}
        self.assertEqual(self.value['packages']['npm:b']['metadata'],{})
        with self.assertRaisesRegex(ValueError,'verified public proof'):self.decode()

    def test_registry_race_keeps_metadata_only_not_unused_candidate_bytes_or_producer(self):
        self.configure({'a':'registry-race','b':'source'})
        self.assertEqual(self.decode(),self.value)
        raced=self.value['packages']['npm:a']
        candidate=next(row for row in self.manifest['packages'] if row['name']=='a')
        self.assertIsNone(self.proof['packages']['npm:a'])
        self.assertFalse(raced['registryBytesVerified'])
        self.assertEqual(raced['originKind'],'registry-race')
        for field in ['archive','archiveSha256','archiveBytes','originSourceSHA','originSourceTree','originRunId','originAttempt']:
            self.assertNotIn(field,raced)
        proof=copy.deepcopy(self.proof);proof['dispositions']['npm:a']['uploadAttempted']=True
        with self.assertRaisesRegex(ValueError,'cannot claim a source upload'):self.decode(proof=proof)
        value=copy.deepcopy(self.value);value['packages']['npm:a'].update(candidate)
        with self.assertRaisesRegex(ValueError,'verified public proof'):self.decode(self.release_for(value))

    def test_source_must_match_the_sealed_bytes_and_actual_upload(self):
        for field,value in [('archiveSha256','f'*64),('archiveBytes',999),('uploadAttempted',False),
                            ('originKind','registry')]:
            proof=copy.deepcopy(self.proof);proof['dispositions']['npm:b'][field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):self.decode(proof=proof)
        proof=copy.deepcopy(self.proof);proof['dispositions']['npm:a']['originKind']='source'
        with self.assertRaises(ValueError):self.decode(proof=proof)

    def test_retained_candidate_cannot_override_published_family(self):
        manifest=copy.deepcopy(self.manifest)
        manifest['packages'].append({**self.original['packages'][0],'archive':'unused.tgz'})
        proof={**self.proof,'manifestHash':module.chain.digest(manifest)}
        with self.assertRaisesRegex(ValueError,'incomplete build or registry evidence'):
            module.create(manifest,self.source,None,proof,1)
        proof=copy.deepcopy(self.proof);proof['dispositions'].pop('npm:a')
        with self.assertRaisesRegex(ValueError,'public family'):self.decode(proof=proof)
        proof=copy.deepcopy(self.proof);proof['dispositions']['npm:a']['archiveSha256']='f'*64
        with self.assertRaisesRegex(ValueError,'archive verification'):self.decode(proof=proof)
        proof=copy.deepcopy(self.proof);proof['dispositions']['npm:a']['registryBytesVerified']=True
        with self.assertRaisesRegex(ValueError,'archive verification'):self.decode(proof=proof)

    def test_registry_only_and_all_raced_receipts_never_invent_source_uploads(self):
        for origins in [{'a':'registry','b':'registry'},{'a':'registry-race','b':'registry-race'}]:
            with self.subTest(origins=origins):
                self.configure(origins)
                self.assertEqual(self.decode(),self.value)
                self.assertFalse(any(row['originKind']=='source' for row in self.value['packages'].values()))
                self.assertTrue(all('originSourceSHA' not in row and 'archive' not in row
                                    for row in self.value['packages'].values()))
                if origins['a']=='registry':self.assertEqual(self.manifest['packages'],[])

    def test_existing_release_confirmation_passes_verified_proof_without_new_writes(self):
        publisher=load('checkpoint_presence_publish','incremental-release-publish.py')
        api=mock.Mock();api.reader.get.return_value={'id':1}
        def request(method,route,**options):
            self.assertEqual(method,'GET','Existing confirmation must be read-only')
            if route.startswith('/git/ref/tags/'):
                return {'object':{'type':'commit','sha':self.source['sha']}}
            return self.release
        api.request.side_effect=request
        with mock.patch.object(publisher.entry.evidence,'source_reader',return_value=self.source), \
             mock.patch.object(publisher,'verify_release_checkpoint',wraps=publisher.verify_release_checkpoint) as verify:
            self.assertTrue(publisher.confirm_release(api,self.manifest,self.proof)['verified'])
            self.assertEqual(verify.call_args.kwargs,{'proof':self.proof})
            with self.assertRaisesRegex(ValueError,'verified public proof'):
                publisher.verify_release_checkpoint(api,self.manifest,self.release)
            forged=copy.deepcopy(self.value);forged['packages']['npm:a']['archiveSha256']='f'*64
            self.release=self.release_for(forged)
            with self.assertRaisesRegex(ValueError,'verified public proof'):
                publisher.confirm_release(api,self.manifest,self.proof)

    def test_new_mixed_release_body_does_not_attribute_retained_versions_to_source(self):
        publisher=load('checkpoint_presence_body','incremental-release-publish.py')
        for origins in [{'a':'registry','b':'source'},{'a':'registry-race','b':'source'}]:
            self.configure(origins)
            api=mock.Mock();api.reader.get.return_value={'id':1}
            state={'ref':None,'release':None};writes=[]
            def request(method,route,data=None,**options):
                if method=='GET':return state['ref'] if route.startswith('/git/') else state['release']
                writes.append((route,data))
                if route=='/git/refs':state['ref']={'object':{'type':'commit','sha':data['sha']}}
                if route=='/releases':
                    state['release']={'id':5,**data,'author':{'login':'github-actions[bot]','type':'Bot'}}
                return {}
            api.request.side_effect=request
            with mock.patch.object(publisher.entry.evidence,'source_reader',return_value=self.source), \
                 mock.patch.object(publisher.entry,'release_checkpoint',return_value=module.encode(self.value)):
                self.assertTrue(publisher.confirm_release(api,self.manifest,self.proof)['verified'])
            body=state['release']['body']
            self.assertIn('Source-built packages: 1',body)
            self.assertIn('Registry-retained packages: 1',body)
            self.assertIn('not rebuilt artifacts from this source',body)
            self.assertIn('does not claim current-source provenance for retained versions',body)
            self.assertEqual([route for route,_ in writes],['/git/refs','/releases'])

    def test_tag_only_finalizer_skips_github_release_for_empty_source_dispositions(self):
        publisher=load('checkpoint_presence_best_effort','release-npm-best-effort.py')
        for origins in [{'a':'registry','b':'registry'},{'a':'registry-race','b':'registry-race'}]:
            self.configure(origins)
            with mock.patch.object(publisher,'new_report'), \
                 mock.patch.object(publisher,'authorize',return_value=self.proof), \
                 mock.patch.object(publisher.publisher,'GitHubWriter') as writer, \
                 mock.patch.object(publisher.publisher,'confirm_release') as confirm:
                result=publisher.finalize_release(self.manifest,Path('/unused'),{})
            self.assertEqual(result,{'verified':True,'releaseCreated':False,'reason':'registry-only-confirmation'})
            writer.assert_not_called();confirm.assert_not_called()

if __name__=='__main__':unittest.main(verbosity=2)
