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
        files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
        files['docs/releases/v2-candidate.json']=json.dumps(family).encode()
        self.source={**identity,'files':files,'modes':fixtures.modes(files),'policyHash':'pinned'}
        self.family=family
        rows=[{**row,'archiveSha256':'e'*64,'archiveBytes':1} for row in module.chain.family_records(family).values()]
        self.manifest={'chain':'release','profile':'release','namespace':'farm-release-v1','sourceSHA':identity['sha'],'sourceTree':identity['tree'],
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
    def test_completed_provenance_reads_real_seal_and_public_zip_then_exact_tag_and_release(self):
        fixture=entry_fixtures.Contracts();fixture.setUp()
        manifest=fixture.manifest;manifest['tag']='v1.0.0';transaction=module.chain.digest(manifest)
        fixture.seal['manifestHash']=transaction;fixture.public['manifestHash']=transaction
        for job in fixture.jobs:job['run_id']=10
        metadata={};bodies={}
        for index,(prefix,record) in enumerate([('farm-release-seal-',fixture.seal),('farm-release-public-acceptance-',fixture.public)],100):
            name=prefix+fixture.source['sha']+'-2';body=io.BytesIO()
            with zipfile.ZipFile(body,'w') as archive:archive.writestr('ledger.json',json.dumps(record))
            raw=body.getvalue();bodies[index]=raw
            metadata[name]={'id':index,'name':name,'workflow_run':{'id':10},'expired':False,'size_in_bytes':len(raw),
                            'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'created_at':'2026-01-01T00:00:30Z'}
        for index,platform in enumerate(['ubuntu-24.04','macos-latest','windows-latest'],200):
            name='farm-release-candidate-consumer-'+fixture.source['sha']+'-'+platform+'-2'
            report={'status':'passed','scratchRemoved':True,'platform':{'os':{'ubuntu-24.04':'linux','macos-latest':'darwin','windows-latest':'win32'}[platform],'arch':'arm64' if platform=='macos-latest' else 'x64','abi':{'ubuntu-24.04':'linux-x64-gnu','macos-latest':'darwin-arm64','windows-latest':'win32-x64-msvc'}[platform]},'releaseBinding':{'manifestHash':transaction,'sourceSHA':fixture.source['sha'],
                    'sourceTree':fixture.source['tree'],'producerRunId':10,'producerAttempt':2,'consumerRunId':10,'consumerAttempt':2}}
            body=io.BytesIO()
            with zipfile.ZipFile(body,'w') as archive:archive.writestr('result.json',json.dumps(report))
            raw=body.getvalue();bodies[index]=raw
            metadata[name]={'id':index,'name':name,'workflow_run':{'id':10},'expired':False,'size_in_bytes':len(raw),
                            'digest':'sha256:'+hashlib.sha256(raw).hexdigest(),'created_at':'2026-01-01T00:00:30Z'}
            job=next(row for row in fixture.jobs if row['name']=='Independent release candidate consumer ('+platform+')')
            job['steps']=[{'name':'Upload independent release candidate acceptance','status':'completed','conclusion':'success',
                           'started_at':'2026-01-01T00:00:00Z','completed_at':'2026-01-01T00:01:00Z'}]
        def get(route):
            if route in ['/actions/runs/10','/actions/runs/10/attempts/2']:return fixture.run
            if '/jobs?' in route:return {'jobs':fixture.jobs}
            if '/artifacts?' in route:
                name=module.evidence.transport_module.urllib.parse.parse_qs(module.evidence.transport_module.urllib.parse.urlsplit(route).query)['name'][0]
                rows=[metadata[name]] if name in metadata else [];return {'total_count':len(rows),'artifacts':rows}
            if route.startswith('/actions/artifacts/'):return next(row for row in metadata.values() if row['id']==int(route.rsplit('/',1)[1]))
            if route=='/git/ref/tags/v1.0.0':return {'object':{'type':'commit','sha':fixture.source['sha']}}
            if route=='/releases/tags/v1.0.0':return {'draft':False,'prerelease':False,'body':'<!-- farm-release-manifest:'+transaction+' -->'}
            raise AssertionError(route)
        transport=mock.Mock();transport.get.side_effect=get;transport.archive.side_effect=lambda number:bodies[number]
        reader=module.ReleaseBaselineReader(transport,self.registry,lambda _:fixture.source)
        self.assertEqual(reader.completed(transaction,{'manifest':manifest},fixture.repository)[0],manifest)
        linux_name='farm-release-candidate-consumer-'+fixture.source['sha']+'-ubuntu-24.04-2'
        original_linux=bodies[200];original_linux_meta=copy.deepcopy(metadata[linux_name])
        for field,value in [('os','darwin'),('abi','linux-x64-musl')]:
            wrong=json.loads(zipfile.ZipFile(io.BytesIO(original_linux)).read('result.json'));wrong['platform'][field]=value
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,'w') as archive:archive.writestr('result.json',json.dumps(wrong))
            bodies[200]=stream.getvalue();metadata[linux_name].update({'size_in_bytes':len(bodies[200]),'digest':'sha256:'+hashlib.sha256(bodies[200]).hexdigest()})
            with self.assertRaises(ValueError):reader.completed(transaction,{'manifest':manifest},fixture.repository)
        bodies[200]=original_linux;metadata[linux_name]=original_linux_meta
        current={**fixture.run,'run_attempt':3};new_jobs=copy.deepcopy(fixture.jobs)
        for job in new_jobs:
            if job['name'].startswith('Independent release candidate consumer (') and 'windows-latest' not in job['name']:job['conclusion']='failure'
        original_windows=next(job for job in fixture.jobs if job['name']=='Independent release candidate consumer (windows-latest)')
        original_windows['conclusion']='failure'
        old_name='farm-release-candidate-consumer-'+fixture.source['sha']+'-windows-latest-2'
        name=old_name[:-1]+'3';report=json.loads(zipfile.ZipFile(io.BytesIO(bodies[202])).read('result.json'))
        report['releaseBinding']['consumerAttempt']=3;body=io.BytesIO()
        with zipfile.ZipFile(body,'w') as archive:archive.writestr('result.json',json.dumps(report))
        raw=body.getvalue();bodies[203]=raw
        metadata[name]={**metadata[old_name],'id':203,'name':name,'size_in_bytes':len(raw),'digest':'sha256:'+hashlib.sha256(raw).hexdigest()}
        def mixed(route):
            if route in ['/actions/runs/10','/actions/runs/10/attempts/3']:return current
            if '/attempts/3/jobs?' in route:return {'jobs':new_jobs}
            return get(route)
        transport.get.side_effect=mixed
        self.assertEqual(reader.completed(transaction,{'manifest':manifest},fixture.repository)[0],manifest)
        # Producer 2, Mac acceptance 2, Linux acceptance 3, Windows acceptance 4.
        # Neither original+current alone nor one shared successful attempt works.
        middle={**current};last={**current,'run_attempt':4}
        middle_jobs=copy.deepcopy(new_jobs)
        for job in middle_jobs:
            if job['name'].startswith('Independent release candidate consumer ('):
                job['conclusion']='success' if 'ubuntu-24.04' in job['name'] else 'failure'
        next(job for job in fixture.jobs if job['name']=='Independent release candidate consumer (ubuntu-24.04)')['conclusion']='failure'
        for index,platform,attempt in [(204,'ubuntu-24.04',3),(205,'windows-latest',4)]:
            original_name='farm-release-candidate-consumer-'+fixture.source['sha']+'-'+platform+'-2'
            original_id=200 if platform=='ubuntu-24.04' else 202
            report=json.loads(zipfile.ZipFile(io.BytesIO(bodies[original_id])).read('result.json'));report['releaseBinding']['consumerAttempt']=attempt
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,'w') as archive:archive.writestr('result.json',json.dumps(report))
            bodies[index]=stream.getvalue();name=original_name[:-1]+str(attempt)
            metadata[name]={**metadata[original_name],'id':index,'name':name,'size_in_bytes':len(bodies[index]),'digest':'sha256:'+hashlib.sha256(bodies[index]).hexdigest()}
        def three_attempts(route):
            if route in ['/actions/runs/10','/actions/runs/10/attempts/4']:return last
            if route=='/actions/runs/10/attempts/3':return middle
            if '/attempts/4/jobs?' in route:return {'jobs':new_jobs}
            if '/attempts/3/jobs?' in route:return {'jobs':middle_jobs}
            return get(route)
        transport.get.side_effect=three_attempts
        self.assertEqual(reader.completed(transaction,{'manifest':manifest},fixture.repository)[0],manifest)
        last['head_sha']='f'*40
        with self.assertRaises(ValueError):reader.completed(transaction,{'manifest':manifest},fixture.repository)
        last['head_sha']=fixture.source['sha']
        next(iter(metadata.values()))['digest']='sha256:'+'0'*64
        with self.assertRaises(ValueError):reader.completed(transaction,{'manifest':manifest},fixture.repository)

if __name__=='__main__':unittest.main(verbosity=2)
