"""Simulated verified-certificate policy tests, NOT live signature validation."""
# cspell:words importlib sigstore
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('attested_policy',Path(__file__).with_name('ci-attested-bindings.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

def fixture():
    e={'repository':'farm-fe/farm','repositoryId':123,'sourceSHA':'a'*40,'sourceTree':'b'*40,'runId':42,'attempt':3,'callerWorkflow':'.github/workflows/ci.yaml','signerWorkflow':'.github/workflows/ci-provenance-signer.yaml','signerSHA':'c'*40,'cells':{'planner':{'producer':'Resolve impact','uploadStep':'Upload plan','artifactName':'plan-{sourceSHA}-{originAttempt}'}}}
    row={'cell':'planner','producer':'Resolve impact','uploadStep':'Upload plan','originAttempt':1,'jobId':456,'artifactId':789,'artifactDigest':'sha256:'+'d'*64,'artifactName':'plan-'+e['sourceSHA']+'-1'}
    manifest={'schemaVersion':1,'kind':'fixed-workflow-upload-bindings',**{k:e[k] for k in ['repositoryId','sourceSHA','sourceTree','runId','callerWorkflow']},'aggregationAttempt':3,'bindings':[row]}
    repo='https://github.com/farm-fe/farm';signer=repo+'/'+e['signerWorkflow']+'@'+e['signerSHA']
    cert={'issuer':'https://token.actions.githubusercontent.com','subjectAlternativeName':signer,'buildSignerURI':signer,'buildSignerDigest':e['signerSHA'],'runnerEnvironment':'github-hosted','sourceRepositoryURI':repo,'sourceRepositoryIdentifier':'123','sourceRepositoryDigest':e['sourceSHA'],'sourceRepositoryRef':'refs/heads/main','buildConfigURI':repo+'/'+e['callerWorkflow']+'@refs/heads/main','buildConfigDigest':e['sourceSHA'],'buildTrigger':'push','runInvocationURI':repo+'/actions/runs/42/attempts/3'}
    body=json.dumps(manifest).encode();statement={'_type':'https://in-toto.io/Statement/v1','predicateType':m.PREDICATE,'subject':[{'name':'ci-bindings.json','digest':{'sha256':hashlib.sha256(body).hexdigest()}}],'predicate':{'schemaVersion':1,'kind':'fixed-workflow-upload-bindings'}}
    return body,[{'verificationResult':{'signature':{'certificate':cert},'statement':statement}}],e

class Contracts(unittest.TestCase):
    def test_valid_policy_preserves_origin_without_clock_claims(self):
        body,verified,e=fixture();self.assertEqual(m.validate_verified_manifest(body,verified,e)['planner']['originAttempt'],1)
    def test_each_certificate_field_missing_or_wrong_rejects(self):
        for key in fixture()[1][0]['verificationResult']['signature']['certificate']:
            for defect in ['missing','wrong']:
                body,verified,e=fixture();cert=verified[0]['verificationResult']['signature']['certificate']
                if defect=='missing':cert.pop(key)
                else:cert[key]='wrong'
                with self.subTest(key=key,defect=defect),self.assertRaises(ValueError):m.validate_verified_manifest(body,verified,e)
    def test_manifest_binding_tamper_rejects_even_with_simulated_valid_subject(self):
        for defect in ['extra','duplicate','job','attempt','producer','step','digest','artifact-name','source']:
            body,verified,e=fixture();value=json.loads(body);row=value['bindings'][0]
            if defect=='extra':row['trusted']=True
            elif defect=='duplicate':value['bindings'].append(copy.deepcopy(row))
            elif defect=='job':row['jobId']=True
            elif defect=='attempt':row['originAttempt']=4
            elif defect=='producer':row['producer']='Other'
            elif defect=='step':row['uploadStep']='Other'
            elif defect=='digest':row['artifactDigest']='bad'
            elif defect=='artifact-name':row['artifactName']='wrong'
            else:value['sourceSHA']='e'*40
            body=json.dumps(value).encode();verified[0]['verificationResult']['statement']['subject'][0]['digest']['sha256']=hashlib.sha256(body).hexdigest()
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.validate_verified_manifest(body,verified,e)
    def test_missing_ambiguous_or_wrong_subject_rejects(self):
        body,verified,e=fixture()
        for values in [[],verified*2,[{}]]:
            with self.assertRaises(ValueError):m.validate_verified_manifest(body,values,e)
        with self.assertRaises(ValueError):m.validate_verified_manifest(body+b' ',verified,e)
    def test_exact_caller_template_and_single_signer_slot(self):
        template={'ci.yaml':b'uses: farm-fe/farm/signer@__REVIEWED_SIGNER_REVISION__','producer.yaml':b'fixed upload outputs'}
        files={k:v.replace(b'__REVIEWED_SIGNER_REVISION__',b'a'*40) for k,v in template.items()}
        m.verify_caller_templates(files,template,'a'*40)
        for key in files:
            changed={**files,key:files[key]+b' arbitrary caller JSON'}
            with self.assertRaises(ValueError):m.verify_caller_templates(changed,template,'a'*40)
        with self.assertRaises(ValueError):m.verify_caller_templates(files,template,'b'*40)

    def test_pinned_verifier_result_is_consumed_directly_and_failures_reject(self):
        import tempfile
        from types import SimpleNamespace
        body,verified,e=fixture()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);cli=root/'gh';cli.write_bytes(b'not-executed-test-fixture')
            manifest=root/'manifest.json';manifest.write_bytes(body);bundle=root/'bundle.json';bundle.write_bytes(b'not-a-real-signature')
            digest=hashlib.sha256(cli.read_bytes()).hexdigest();calls=[]
            def invoke(command, **options):
                calls.append((command,options))
                return SimpleNamespace(returncode=0,stdout=json.dumps(verified).encode())
            self.assertEqual(len(m.verify_with_pinned_cli(cli,digest,manifest,bundle,e,invoke=invoke)),1)
            command,options=calls[0]
            self.assertEqual(command[1:3],['attestation','verify'])
            self.assertIn('--deny-self-hosted-runners',command);self.assertIn('--cert-identity',command)
            # gh 2.103.3 accepts exactly one certificate identity selector.
            selectors = ['--cert-identity', '--cert-identity-regex', '--signer-repo', '--signer-workflow']
            self.assertEqual([flag for flag in selectors if flag in command], ['--cert-identity'])
            for flag, value in {
                '--cert-identity': 'https://github.com/' + e['repository'] + '/' + e['signerWorkflow'] + '@' + e['signerSHA'],
                '--signer-digest': e['signerSHA'], '--source-digest': e['sourceSHA'],
                '--source-ref': 'refs/heads/main', '--repo': e['repository'],
                '--predicate-type': m.PREDICATE,
                '--cert-oidc-issuer': 'https://token.actions.githubusercontent.com',
            }.items():
                self.assertEqual(command[command.index(flag) + 1], value)
            self.assertNotIn('GH_TOKEN',options['env']);self.assertNotIn('GITHUB_TOKEN',options['env'])
            for code in [1,2]:
                with self.assertRaises(ValueError):
                    m.verify_with_pinned_cli(cli,digest,manifest,bundle,e,invoke=lambda *a,**k:SimpleNamespace(returncode=code,stdout=json.dumps(verified).encode()))
            with self.assertRaises(ValueError):m.verify_with_pinned_cli(cli,'0'*64,manifest,bundle,e,invoke=invoke)

    @unittest.skipUnless(os.environ.get('FARM_ATTESTATION_CLI_FIXTURE'), 'Explicit real signed CLI fixture required')
    def test_real_pinned_cli_accepts_original_cargo_proof_and_rejects_tampering(self):
        # Opt-in local integration: existing public proof + official CLI archive.
        # No signing, token, registry writes or fixture JSON verification bypass.
        import tempfile
        import zipfile
        root = Path(os.environ['FARM_ATTESTATION_CLI_FIXTURE'])
        archive = root / 'cargo-proof.zip'
        self.assertEqual(hashlib.sha256(archive.read_bytes()).hexdigest(),
                         '0475cb303c47358951387bb8478fed32cde74148e0c1d07d12595769aba64a11')
        spec = importlib.util.spec_from_file_location('real_cli_assets', Path(__file__).with_name('ci-pinned-attestation-cli.py'))
        assets = importlib.util.module_from_spec(spec); spec.loader.exec_module(assets)
        import platform
        import sys
        name, size, digest, member = assets.host_asset(sys.platform, platform.machine())
        cli_archive = (root / name).read_bytes()
        self.assertEqual(len(cli_archive), size)
        binary = assets.unpack_verified_binary(cli_archive, digest, member, zipped=name.endswith('.zip'))
        expected = {'repository': 'farm-fe/farm', 'repositoryId': 507542208,
                    'sourceSHA': 'b03f831d9f0a336ef6c0121adcf4891f7bf8e959',
                    'sourceTree': '6c9a6ce3bd30d24cc82883d73fb0bf705e22c422',
                    'runId': 42730979843, 'attempt': 1,
                    'callerWorkflow': '.github/workflows/cargo-source-release.yaml',
                    'signerWorkflow': '.github/workflows/release-provenance-signer.yaml',
                    'signerSHA': '9f93b5e5359bbf0c3d7f750f7a24469eeaab9b6d',
                    'cells': {'cargo': {'producer': 'Prepare Cargo source archives',
                              'uploadStep': 'Freeze verified Cargo source archives',
                              'artifactName': 'cargo-source-{runId}'}}}
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory); cli = output / ('gh.exe' if sys.platform == 'win32' else 'gh')
            cli.write_bytes(binary); cli.chmod(0o700)
            manifest = output / 'ci-bindings.json'; bundle = output / 'ci-bindings.sigstore.json'
            with zipfile.ZipFile(archive) as proof:
                manifest.write_bytes(proof.read(manifest.name)); bundle.write_bytes(proof.read(bundle.name))
            def verify(policy):
                return m.verify_with_pinned_cli(cli, hashlib.sha256(binary).hexdigest(), manifest, bundle, policy)
            self.assertEqual(verify(expected)['cargo']['artifactId'], 13724513300)
            for field, value in [('sourceSHA', '0' * 40), ('signerSHA', '0' * 40),
                                 ('signerWorkflow', '.github/workflows/other.yaml')]:
                with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'Cryptographic'):
                    verify({**expected, field: value})
            manifest.write_bytes(manifest.read_bytes() + b' ')
            with self.assertRaisesRegex(ValueError, 'Cryptographic'): verify(expected)

    def test_default_process_path_rejects_forged_verified_json_on_failed_verification(self):
        import os
        import sys
        import tempfile
        from unittest.mock import patch
        body,verified,e=fixture()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);manifest=root/'manifest.json';manifest.write_bytes(body)
            bundle=root/'bundle.json';bundle.write_bytes(json.dumps(verified).encode())
            forged=root/'verified-result.json';forged.write_bytes(json.dumps(verified).encode())
            # An actual local subprocess, not invoke injection. It emits a
            # policy-shaped forgery but signals failed cryptographic validation.
            cli=root/'rejected-verifier'
            cli.write_text("#!/bin/sh\ncat <<'FIXTURE_JSON'\n"+json.dumps(verified)+"\nFIXTURE_JSON\nexit 1\n")
            cli.chmod(0o700)
            with patch.dict(os.environ,{'FARM_VERIFIED_RESULT':str(forged),'GH_ATTESTATION_VERIFIED_RESULT':str(forged)}):
                with self.assertRaisesRegex(ValueError,'Cryptographic attestation verification failed'):
                    m.verify_with_pinned_cli(cli,hashlib.sha256(cli.read_bytes()).hexdigest(),manifest,bundle,e)

    def test_signer_builder_fetches_exact_caller_and_original_api_bindings(self):
        import base64
        _,_,e=fixture();template={e['callerWorkflow']:b'uses: signer@__REVIEWED_SIGNER_REVISION__'}
        source=template[e['callerWorkflow']].replace(b'__REVIEWED_SIGNER_REVISION__',e['signerSHA'].encode())
        run={'id':42,'run_attempt':3,'repository':{'id':123,'full_name':'farm-fe/farm'},'head_repository':{'id':123},'head_sha':e['sourceSHA'],'head_branch':'main','event':'push','path':e['callerWorkflow']}
        artifact={'id':789,'name':'plan-'+e['sourceSHA']+'-1','workflow_run':{'id':42},'expired':False,'digest':'sha256:'+'d'*64}
        output={'planner':{'artifact_id':'789','artifact_digest':'d'*64,'run_attempt':'1'}}
        state={'bad':None};calls=[]
        class API:
            def get(self,route):
                calls.append(route)
                if '/attempts/' in route:
                    attempt=int(route.split('/attempts/')[1].split('/')[0])
                    if '/jobs?' in route:
                        return {'jobs':[{'id':456 if attempt==1 else 456+attempt,'run_id':42,'head_sha':e['sourceSHA'],'run_attempt':attempt,'name':'Resolve impact','status':'completed','conclusion':'failure' if state['bad']=='latest' and attempt==3 else 'success','steps':[{'name':'Upload plan','status':'completed','conclusion':'success'}]}]}
                    return {**run,'run_attempt':attempt}
                if '/git/commits/' in route:return {'sha':e['sourceSHA'],'tree':{'sha':e['sourceTree']}}
                if '/contents/' in route:
                    body=source+(b' modified caller' if state['bad']=='caller' else b'')
                    return {'type':'file','path':e['callerWorkflow'],'encoding':'base64','size':len(body),'content':base64.b64encode(body).decode()}
                return {**artifact,'digest':'sha256:'+'e'*64} if state['bad']=='digest' else artifact
        body=m.build_manifest_from_fixed_outputs(API(),e,output,template)
        value=json.loads(body);self.assertEqual(value['bindings'][0]['jobId'],456)
        self.assertEqual(value['bindings'][0]['originAttempt'],1)
        self.assertTrue(any('?ref='+e['sourceSHA'] in route for route in calls))
        for defect in ['caller','latest','digest']:
            state['bad']=defect
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.build_manifest_from_fixed_outputs(API(),e,output,template)

    def test_verified_binding_consumer_reads_exact_original_bytes_without_clocks(self):
        body,verified,e=fixture();manifest=json.loads(body);payload=b'actual isolated archive bytes'
        row=manifest['bindings'][0];row['artifactDigest']='sha256:'+hashlib.sha256(payload).hexdigest()
        body=json.dumps(manifest).encode()
        verified[0]['verificationResult']['statement']['subject'][0]['digest']['sha256']=hashlib.sha256(body).hexdigest()
        accepted=m.validate_verified_manifest(body,verified,e)['planner']
        run={'id':42,'run_attempt':3,'repository':{'id':123,'full_name':'farm-fe/farm'},'head_repository':{'id':123},'head_sha':e['sourceSHA'],'head_branch':'main','event':'push','path':e['callerWorkflow']}
        state={'defect':None};downloads=[]
        class API:
            def get(self,route):
                if '/attempts/' in route:
                    attempt=int(route.split('/attempts/')[1].split('/')[0])
                    if '/jobs?' in route:
                        job={'id':456 if attempt==1 else 456+attempt,'run_id':42,'head_sha':e['sourceSHA'],'run_attempt':attempt,'name':'Resolve impact','status':'completed','conclusion':'success','steps':[{'name':'Upload plan','status':'completed','conclusion':'success'}]}
                        if state['defect']=='job' and attempt==1:job['id']=999
                        if state['defect']=='latest' and attempt==3:job['conclusion']='failure'
                        if state['defect']=='step':job['steps'][0]['conclusion']='failure'
                        if state['defect']=='source':job['head_sha']='e'*40
                        return {'jobs':[job]}
                    return {**run,'run_attempt':attempt}
                artifact={'id':789,'name':row['artifactName'],'workflow_run':{'id':42},'expired':False,'digest':row['artifactDigest'],'size_in_bytes':len(payload)}
                if state['defect']=='artifact-id':artifact['id']=790
                if state['defect']=='digest':artifact['digest']='sha256:'+'0'*64
                if state['defect']=='run':artifact['workflow_run']['id']=43
                return artifact
            def archive(self,artifact_id):
                downloads.append(artifact_id)
                return payload+b'tamper' if state['defect']=='bytes' else payload
        self.assertEqual(m.bound_artifact(API(),run,accepted,e)[0],payload)
        self.assertEqual(downloads,[789])
        for later in [4,5]:
            with self.assertRaisesRegex(ValueError,'new attempt requires'):
                m.bound_artifact(API(),{**run,'run_attempt':later},accepted,e)
        self.assertEqual(downloads,[789])
        for defect in ['job','latest','step','source','artifact-id','digest','run','bytes']:
            state['defect']=defect
            with self.subTest(defect=defect),self.assertRaises(ValueError):m.bound_artifact(API(),run,accepted,e)

if __name__=='__main__':unittest.main()
