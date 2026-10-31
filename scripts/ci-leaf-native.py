"""Live CI-only native byte resolution for engineering-documentation leaves.

No release bytes, inherited test results, credentials or mutable cache entries
are accepted. Only Linux GNU is enabled; every other ABI still builds and all
current TS/Examples/performance suites still execute.
"""
# cspell:words argparse importlib tomllib IFMT IFREG RUSTFLAGS
import argparse
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tomllib
import zipfile


def load(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value


evidence=load('leaf_chain_evidence','incremental-chain-evidence.py')
proof=load('leaf_native_paths','ci-native-proof.py')
chain=evidence.chain_plan
check=evidence.check
ABI='linux-x64-gnu'
IMAGE='ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian'
WORKFLOW='.github/workflows/ci.yaml'
UPLOAD='Upload trusted CI native byte bundle'
SUFFIX='Build and Upload User Artifacts - '+ABI
MAX=512*1024*1024


def normal_file(root,name):
    proof.safe_member(name);base=Path(root).resolve(strict=True);current=base
    for part in name.split('/'):
        current=current/part
        check(not current.is_symlink() and not current.is_junction() and current.resolve(strict=True).is_relative_to(base),'Native path redirected')
    check(current.is_file(),'Native regular file required');return current


def gnu_header(body):
    check(len(body)>=20 and body[:4]==b'\x7fELF' and body[4:6]==b'\x02\x01' and int.from_bytes(body[18:20],'little')==62,'Wrong native binary format or architecture')


def leaf(file, mode):
    return file.startswith('docs/experiences/') and file.endswith('.md') and mode=='100644'


def inputs(source):
    return chain.digest({name:row for name,row in chain.snapshot_records(source['files'],source['modes']).items() if not leaf(name,row['mode'])})


def farm_abi(source):
    found=re.search(r'^pub const VERSION: &str = "([0-9]+\.[0-9]+\.[0-9]+)";',source['files']['crates/core/src/lib.rs'].decode('utf-8'),re.M)
    check(found is not None,'Exact Farm plugin ABI declaration required');return found[1]


def products(source):
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Unresolved native source graph')
    family=chain.independent_family(source['files'],nodes);result={}
    for key,row in family.items():
        if row.get('platform')!=ABI:continue
        owner=family['npm:'+row['owner']];directory=str(Path(owner['manifestPath']).parent).replace('\\','/')
        file=('packages/core/binding/farm.'+ABI+'.node' if row['owner']=='@farmfe/core' else
              'packages/create-farm/create-farm.'+ABI+'.node' if row['owner']=='create-farm' else directory+'/npm/'+ABI+'/index.farm')
        result[row['name']]={'file':file,'owner':row['owner'],'version':row['version']}
    check(result,'Complete native cohort required')
    return result


def requested_flags():
    plugins='react replace-dirname sass vue vue-jsx auto-import compress dsv dts icons image mdx modular-import react-components strip svgr url virtual wasm worker yaml tailwindcss ssr-css syntax-highlight docs docs-snapshot docs-seo docs-snippets'
    values={'RUSTFLAGS':None,'CARGO_ENCODED_RUSTFLAGS':None,'CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':None,
            'CARGO_PROFILE_RELEASE_LTO':None,'CARGO_PROFILE_RELEASE_CODEGEN_UNITS':None,
            'FARM_BUILD_PROFILE':'ci','FARM_BUILD_ABI':ABI,'FARM_BUILD_TARGET':'x86_64-unknown-linux-gnu','FARM_BUILD_ZIG':'false',
            'FARM_CAPTURE_CORE_SCRIPT':'build:rs:ci','FARM_CAPTURE_CORE_ARGS':'--target x86_64-unknown-linux-gnu  ',
            'FARM_CAPTURE_PLUGIN_ARGS':'--target x86_64-unknown-linux-gnu  --abi linux-x64-gnu --profile ci',
            'FARM_CAPTURE_CREATE_SCRIPT':'build:ci','FARM_CAPTURE_PLUGINS':plugins}
    return hashlib.sha256(json.dumps(values,separators=(',',':')).encode()).hexdigest()


def context(source, captured, image):
    runtime=captured.get('runtimeCapture',{})
    channel=tomllib.loads(source['files']['rust-toolchain.toml'].decode('utf-8'))['toolchain']['channel']
    check(runtime.get('profile')=='ci' and runtime.get('abi')==ABI and runtime.get('target')=='x86_64-unknown-linux-gnu','Measured CI ABI/profile differs')
    check(runtime.get('errors')==[] and runtime.get('linker',{}).get('version') and runtime.get('rustc') and runtime.get('cargo'),'Actual compiler/linker probes missing')
    check(runtime.get('runtime',{}).get('node')=='v22.20.0','Actual native build Node version differs')
    check(runtime.get('activeToolchain','').split(' ')[0].startswith(channel+'-'),'Actual Rust toolchain differs from pinned policy')
    check(runtime.get('flagsHash')==requested_flags() and runtime.get('unboundBuildVariables')==[],'Measured build flags or environment differ from fixed GNU CI recipe')
    check(re.fullmatch('sha256:[a-f0-9]{64}',image or '') and captured.get('container',{}).get('containerImageId')==image,'Exact container image differs')
    check(captured['container'].get('buildExitCode')==0 and captured['container'].get('cleanupConfirmed') is True
          and captured['container'].get('timedOut') is False,'Native producer did not finish and clean up successfully')
    return {'profile':'ci','abi':ABI,'farmABI':farm_abi(source),'target':'x86_64-unknown-linux-gnu','imageId':image,'toolchainPolicy':channel,
            'activeToolchain':runtime['activeToolchain'].split(' ')[0],'rustc':runtime['rustc'],'cargo':runtime['cargo'],
            'linker':runtime['linker'],'flagsHash':runtime['flagsHash'],'node':'v22.20.0','inputHash':inputs(source)}


def create_bundle(source,captured,root,environment):
    check(environment.get('GITHUB_EVENT_NAME')=='push' and environment.get('GITHUB_REF')=='refs/heads/main'
          and environment.get('GITHUB_WORKFLOW_REF')==environment.get('GITHUB_REPOSITORY','')+'/'+WORKFLOW+'@refs/heads/main','Only trusted main CI produces baselines')
    check(environment.get('GITHUB_SHA')==source['sha'] and captured.get('originBuildSHA')==source['sha'],'Producer checkout differs')
    wanted=products(source);rows=captured.get('records',[])
    check(len(rows)==len(wanted) and {row['name'] for row in rows}==set(wanted),'Incomplete native baseline')
    image=captured.get('container',{}).get('containerImageId');measured=context(source,captured,image)
    records=[];bodies={}
    for index,row in enumerate(sorted(rows,key=lambda row:row['name'])):
        check(row['file']==wanted[row['name']]['file'],'Producer destination differs')
        file=normal_file(root,row['file'])
        body=file.read_bytes();check(len(body)==row['bytes'] and hashlib.sha256(body).hexdigest()==row['sha256'],'Producer byte observation differs')
        gnu_header(body)
        member='products/'+str(index)+'.bin';bodies[member]=body
        records.append({**wanted[row['name']],'name':row['name'],'member':member,'bytes':len(body),'sha256':row['sha256']})
    check(len(records)<64 and sum(len(body) for body in bodies.values())<=MAX-256*1024,'Native bundle exceeds declared quota bound')
    return {'schemaVersion':1,'kind':'ci-native-build-bytes','chain':'ci','context':measured,'sourceSHA':source['sha'],'sourceTree':source['tree'],
            'runId':int(environment['GITHUB_RUN_ID']),'attempt':int(environment['GITHUB_RUN_ATTEMPT']),'records':records},bodies


def inspect_bundle(body,artifact,run,job,current,origin,image):
    check(run['path']==WORKFLOW and run['event']=='push' and run['head_branch']=='main' and run['status']=='completed','Only completed main CI producer accepted')
    check(job['status']=='completed' and job['conclusion']=='success' and job['run_id']==run['id'] and job['name'].endswith(SUFFIX),'Actual CI producer did not succeed')
    steps=[step for step in job.get('steps',[]) if step.get('name')==UPLOAD]
    check(len(steps)==1 and steps[0]['conclusion']=='success' and steps[0]['started_at']<=artifact['created_at']<=steps[0]['completed_at'],'Bundle upload is not its successful producer step')
    check(artifact['workflow_run']['id']==run['id'] and artifact['expired'] is False and artifact['name']=='farm-ci-byte-bundle-'+run['head_sha']+'-'+ABI+'-'+str(run['run_attempt']),'Bundle artifact identity differs')
    check(0<len(body)<=MAX and len(body)==artifact['size_in_bytes'] and artifact['digest']=='sha256:'+hashlib.sha256(body).hexdigest(),'Bundle archive digest differs')
    check(origin['sha']==run['head_sha'] and inputs(origin)==inputs(current),'Current native inputs differ')
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries=archive.infolist();check(0<len(entries)<=64 and sum(row.file_size for row in entries)<=MAX,'Bundle size/member bound exceeded')
        proof.validate_member_set([row.filename for row in entries],ABI)
        check(all(not row.is_dir() and stat.S_IFMT(row.external_attr>>16) in [0,stat.S_IFREG] for row in entries),'Bundle special member rejected')
        check(archive.getinfo('bundle.json').file_size<=256*1024,'Bundle manifest exceeds bound')
        manifest=json.loads(archive.read('bundle.json').decode('utf-8'))
        check(manifest.get('schemaVersion')==1 and manifest.get('kind')=='ci-native-build-bytes' and manifest.get('chain')=='ci','Legacy/cross-chain bundle rejected')
        check(manifest['sourceSHA']==origin['sha'] and manifest['sourceTree']==origin['tree'] and manifest['runId']==run['id'] and manifest['attempt']==run['run_attempt'],'Bundle source/attempt differs')
        measured=manifest['context'];check(measured['profile']=='ci' and measured['abi']==ABI and measured['farmABI']==farm_abi(current) and measured['target']=='x86_64-unknown-linux-gnu'
              and measured['node']=='v22.20.0' and measured['imageId']==image and measured['inputHash']==inputs(current),'Requested CI context differs')
        check(measured.get('flagsHash')==requested_flags(),'Current GNU CI flags differ')
        channel=tomllib.loads(current['files']['rust-toolchain.toml'].decode())['toolchain']['channel']
        check(measured['toolchainPolicy']==channel and measured['activeToolchain'].startswith(channel+'-') and measured['linker'].get('version') and measured['rustc'] and measured['cargo'],'Incomplete or wrong measured toolchain')
        wanted=products(current);records=manifest['records']
        check(len(records)==len(wanted) and {row['name'] for row in records}==set(wanted),'Complete native inventory differs')
        check({row.filename for row in entries}=={'bundle.json'}|{row['member'] for row in records},'Unexpected bundle members')
        output={}
        for row in records:
            check(all(row[key]==value for key,value in wanted[row['name']].items()),'Native owner/version/destination differs')
            data=archive.read(proof.safe_member(row['member']))
            check(len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],'Native member bytes differ')
            gnu_header(data)
            output[row['file']]=data
    return manifest,output


def docker_image():
    subprocess.run(['docker','pull',IMAGE],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=180)
    result=subprocess.run(['docker','image','inspect','--format','{{.Id}}',IMAGE],check=True,capture_output=True,text=True,timeout=30)
    value=result.stdout.strip();check(re.fullmatch('sha256:[a-f0-9]{64}',value),'Current Docker image identity missing');return value


def resolve(transport,current,trusted,environment,read_source=evidence.source_reader,measure_image=docker_image):
    check(inputs(current)==inputs(trusted),'Unreviewed/non-leaf current inputs')
    differences={name for name in set(current['files'])|set(trusted['files']) if current['files'].get(name)!=trusted['files'].get(name)
                 or current['modes'].get(name)!=trusted['modes'].get(name)}
    check((differences or environment.get('GITHUB_EVENT_NAME')=='push') and all(leaf(name,current['modes'].get(name,trusted['modes'].get(name))) for name in differences),'Only explicit engineering documentation leaf changes are enabled')
    repository=transport.get('');check(repository['id']==int(environment['GITHUB_REPOSITORY_ID']),'Current repository differs')
    check(repository.get('default_branch')=='main','Trusted default branch differs')
    runs=transport.get('/actions/workflows/ci.yaml/runs?per_page=3&status=completed&event=push&branch=main')['workflow_runs']
    image=None
    for summary in runs[:3]:
        run=transport.get('/actions/runs/'+str(summary['id'])+'/attempts/'+str(summary['run_attempt']))
        if run['repository']['id']!=repository['id'] or run['head_repository']['id']!=repository['id']:continue
        origin=read_source(run['head_sha'])
        if inputs(origin)!=inputs(current):continue
        changed={name for name in set(current['files'])|set(origin['files']) if current['files'].get(name)!=origin['files'].get(name) or current['modes'].get(name)!=origin['modes'].get(name)}
        if not changed or not all(leaf(name,current['modes'].get(name,origin['modes'].get(name))) for name in changed):continue
        execution=load('leaf_original_native_attempt','ci-execution-evidence.py')
        producer_run,producer_job=execution.actual_producer(transport,run,SUFFIX)
        name='farm-ci-byte-bundle-'+run['head_sha']+'-'+ABI+'-'+str(producer_run['run_attempt'])
        listed=transport.get('/actions/runs/'+str(run['id'])+'/artifacts?per_page=2&name='+name)
        if listed.get('total_count')!=1:continue
        artifact=transport.get('/actions/artifacts/'+str(listed['artifacts'][0]['id']))
        if artifact.get('expired') is not False or not 0<artifact.get('size_in_bytes',0)<=MAX:continue
        if image is None:image=measure_image()
        manifest,bodies=inspect_bundle(transport.archive(artifact['id']),artifact,producer_run,producer_job,current,origin,image)
        check(transport.get('/branches/main')['commit']['sha']==trusted['sha'],'Trusted main changed during resolution')
        receipt={'kind':'ci-consumer-materialization-not-a-new-build','chain':'ci','profile':'ci','abi':ABI,'originBuildSHA':manifest['sourceSHA'],
                 'originTree':manifest['sourceTree'],'originRunId':run['id'],'originAttempt':producer_run['run_attempt'],'originArtifactId':artifact['id'],
                 'originArtifactDigest':artifact['digest'],'originContext':manifest['context'],'consumerSHA':current['sha'],'consumerTree':current['tree'],
                 'runId':int(environment['GITHUB_RUN_ID']),'attempt':int(environment['GITHUB_RUN_ATTEMPT']),'inputHash':inputs(current),
                 'records':manifest['records'],'acceptanceInherited':False,'nativeBuildMatrix':[abi for abi in chain.impact.TARGETS if abi!=ABI],
                 'tsMatrix':chain.impact.PRIMARY,'examplesMatrix':chain.impact.PRIMARY}
        receipt['resolutionHash']=chain.digest(receipt)
        return receipt,bodies
    raise ValueError('No complete trusted same-chain producer bundle')


def schedule(transport,current,trusted,environment,read_source=evidence.source_reader,measure_image=docker_image):
    decision={'nativeMode':'build','nativeBuildMatrix':list(chain.impact.TARGETS),'tsMatrix':list(chain.impact.PRIMARY),
              'examplesMatrix':list(chain.impact.PRIMARY),'acceptanceInherited':False,'reason':'evidence-unavailable-build'}
    try:
        receipt,bodies=resolve(transport,current,trusted,environment,read_source,measure_image)
    except (ValueError,KeyError,OSError,PermissionError,subprocess.SubprocessError,UnicodeError,zipfile.BadZipFile):
        return decision,None,{}
    decision.update({'nativeMode':'reuse','nativeBuildMatrix':receipt['nativeBuildMatrix'],'reason':'verified-ci-gnu-byte-reuse'})
    return decision,receipt,bodies


def check_execution(mode,outcomes):
    expected={'type-check','primary-linux','primary-macos','primary-windows','call-rust-build-release',
              'check-native-candidate-assembly','examples-test','ts-test'}
    check(set(outcomes)==expected and all(row.get('result')=='success' for row in outcomes.values()),'Missing, skipped, cancelled or failed actual execution')
    check(mode in ['build','reuse'],'Unknown current native disposition')
    return {'nativeBuildMatrix':[abi for abi in chain.impact.TARGETS if mode!='reuse' or abi!=ABI],
            'tsMatrix':list(chain.impact.PRIMARY),'examplesMatrix':list(chain.impact.PRIMARY),'acceptanceInherited':False}


def materialize(receipt,bodies,directory):
    target=Path(directory);target.mkdir(parents=True,exist_ok=False)
    for file,body in bodies.items():
        proof.safe_member(file);path=target/file;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(body)
    (target/'consumer.json').write_bytes((json.dumps(receipt,indent=2)+'\n').encode('utf-8'))


def consumer_producer(transport, source, environment, commitment):
    """Verify original same-run materialization; no new build/test acceptance."""
    execution = load('leaf_consumer_attempts', 'ci-execution-evidence.py')
    run_id, attempt = int(environment['GITHUB_RUN_ID']), int(environment['GITHUB_RUN_ATTEMPT'])
    run = transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
    check(run['id'] == run_id and run['run_attempt'] == attempt and run['path'] == WORKFLOW
          and run['repository']['id'] == int(environment['GITHUB_REPOSITORY_ID']) == run['head_repository']['id']
          and run['event'] == environment['GITHUB_EVENT_NAME'], 'Materialization current execution differs')
    check(run['head_sha'] == source['sha'] or run['event'] == 'pull_request' and run['head_sha'] in source.get('parents', []), 'Materialization current source differs')
    original, job = execution.actual_producer(transport, run, 'Type Check & Changesets')
    name = 'ci-consumer-native-' + ABI + '-' + str(run_id) + '-' + str(original['run_attempt'])
    listed = transport.get('/actions/runs/' + str(run_id) + '/artifacts?per_page=2&name=' + name)
    check(listed.get('total_count') == 1 and len(listed['artifacts']) == 1, 'Original consumer materialization missing or ambiguous')
    artifact = transport.get('/actions/artifacts/' + str(listed['artifacts'][0]['id']))
    check(artifact['name'] == name and artifact['expired'] is False and artifact['workflow_run']['id'] == run_id
          and 0 < artifact['size_in_bytes'] <= MAX, 'Original consumer artifact differs')
    execution.upload_binding(job, 'Upload current CI consumer native materialization', artifact)
    body = transport.archive(artifact['id'])
    check(len(body) == artifact['size_in_bytes'] and artifact['digest'] == 'sha256:' + hashlib.sha256(body).hexdigest(), 'Original materialization archive digest differs')
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        rows = archive.infolist(); names = [row.filename for row in rows]
        check(len(rows) <= 256 and len(names) == len(set(names)) and sum(row.file_size for row in rows) <= MAX, 'Materialization archive exceeds member bound')
        for row in rows:
            proof.safe_member(row.filename.rstrip('/'))
            check(not row.flag_bits & 1 and stat.S_IFMT(row.external_attr >> 16) in [0, stat.S_IFREG, stat.S_IFDIR], 'Materialization special member rejected')
        check(0 < archive.getinfo('consumer.json').file_size <= 256 * 1024, 'Materialization receipt exceeds bound')
        raw = archive.read('consumer.json'); receipt = json.loads(raw.decode('utf-8'))
        check(hashlib.sha256(raw).hexdigest() == commitment, 'Original materialization commitment differs')
        check(receipt['consumerSHA'] == source['sha'] and receipt['consumerTree'] == source['tree'] and receipt['runId'] == run_id
              and receipt['attempt'] == original['run_attempt'] and receipt['inputHash'] == inputs(source), 'Original materialization source or attempt differs')
        check(receipt['chain'] == 'ci' and receipt['profile'] == 'ci' and receipt['abi'] == ABI and receipt['acceptanceInherited'] is False, 'Original materialization profile differs')
        expected = {'consumer.json'} | {row['file'] for row in receipt['records']}
        regular = {row.filename for row in rows if not row.is_dir()}
        check(regular == expected, 'Materialization file inventory differs')
        files = {name:archive.read(name) for name in regular}
        wanted = products(source)
        check(len(receipt['records']) == len(wanted) and {row['name'] for row in receipt['records']} == set(wanted), 'Original materialization complete family differs')
        measured = receipt['originContext']
        check(measured.get('profile') == 'ci' and measured.get('abi') == ABI and measured.get('farmABI') == farm_abi(source)
              and measured.get('target') == 'x86_64-unknown-linux-gnu' and measured.get('node') == 'v22.20.0'
              and measured.get('flagsHash') == requested_flags() and measured.get('inputHash') == inputs(source), 'Original materialization recipe or inputs differ')
        for row in receipt['records']:
            check(all(row[key] == value for key,value in wanted[row['name']].items()), 'Original materialization product route differs')
            data = files[row['file']]
            check(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], 'Original materialization product bytes differ')
            gnu_header(data)
    return receipt, files, {'resolutionAttempt':original['run_attempt'], 'consumerAttempt':attempt, 'jobId':job['id'], 'artifactId':artifact['id'], 'artifactDigest':artifact['digest'], 'acceptanceInherited':False}


def restore_materialization(transport, source, environment, commitment, destination):
    receipt, files, binding = consumer_producer(transport, source, environment, commitment)
    root = Path(destination); check(not root.exists(), 'Refuse stale materialization destination')
    root.mkdir(parents=True)
    for name, body in files.items():
        target=root/name;target.parent.mkdir(parents=True,exist_ok=True)
        with target.open('xb') as stream:stream.write(body)
    return binding


def consume(directory,expected_hash,source,environment,destination,pool=False):
    root=Path(directory).resolve(strict=True);raw=(root/'consumer.json').read_bytes();receipt=json.loads(raw.decode('utf-8'))
    check(hashlib.sha256(raw).hexdigest()==expected_hash and receipt['resolutionHash']==chain.digest({key:value for key,value in receipt.items() if key!='resolutionHash'}),'Consumer materialization commitment differs')
    check(receipt['kind']=='ci-consumer-materialization-not-a-new-build' and receipt['chain']=='ci' and receipt['profile']=='ci' and receipt['abi']==ABI,'Foreign/cross-chain materialization')
    check(receipt['consumerSHA']==source['sha'] and receipt['consumerTree']==source['tree'] and receipt['inputHash']==inputs(source)
          and receipt['runId']==int(environment['GITHUB_RUN_ID']),'Consumer source/run differs')
    if receipt['attempt'] != int(environment['GITHUB_RUN_ATTEMPT']):
        check(0 < receipt['attempt'] < int(environment['GITHUB_RUN_ATTEMPT']), 'Consumer original attempt differs')
        check(environment.get('GH_TOKEN'), 'Consumer source/run/attempt requires independent original producer proof')
        transport=evidence.transport_module.GitHubReadOnly(environment['GITHUB_REPOSITORY'],environment['GH_TOKEN'],allow_archive=True)
        original, files, binding=consumer_producer(transport,source,environment,expected_hash)
        check(original == receipt and files['consumer.json'] == raw, 'Consumer original receipt differs')
    check(receipt['nativeBuildMatrix']==[abi for abi in chain.impact.TARGETS if abi!=ABI]
          and receipt['tsMatrix']==chain.impact.PRIMARY and receipt['examplesMatrix']==chain.impact.PRIMARY
          and receipt['acceptanceInherited'] is False,'Incomplete or altered actual execution inventory')
    wanted=products(source);check(len(receipt['records'])==len(wanted) and {row['name'] for row in receipt['records']}==set(wanted),'Consumer native inventory differs')
    check(not Path(destination).is_symlink() and not Path(destination).is_junction(),'Consumer root redirects')
    base=Path(destination).resolve(strict=True)
    for row in receipt['records']:
        check(all(row[key]==value for key,value in wanted[row['name']].items()),'Consumer native route differs')
        body=normal_file(root,row['file']).read_bytes();check(len(body)==row['bytes'] and hashlib.sha256(body).hexdigest()==row['sha256'],'Materialized native bytes differ')
        if pool:
            owner=row['owner'];directory=str(Path(row['file']).parent).replace('\\','/')
            suffix='' if owner=='@farmfe/core' else '-create-farm' if owner=='create-farm' else '-plugin-'+directory.split('/')[1]
            target=base/(source['sha']+'-'+ABI+suffix)/Path(row['file']).name
        else:
            target=base/row['file']
        relative=target.relative_to(base)
        current=base
        for part in relative.parts[:-1]:
            current=current/part;current.mkdir(exist_ok=True)
            check(not current.is_symlink() and not current.is_junction() and current.resolve(strict=True).is_relative_to(base),'Consumer destination redirects')
        check(not target.is_symlink() and not target.is_junction() and not target.exists(),'Do not overwrite or follow pre-existing consumer native paths')
        with target.open('xb') as output:output.write(body)
    return receipt


def main():
    parser=argparse.ArgumentParser();sub=parser.add_subparsers(dest='stage',required=True)
    producer=sub.add_parser('produce');producer.add_argument('--products',required=True);producer.add_argument('--output',required=True)
    lookup=sub.add_parser('resolve');lookup.add_argument('--output',required=True);lookup.add_argument('--plan-output',default='ci-native-execution-plan.json')
    restore=sub.add_parser('restore');restore.add_argument('--output',required=True);restore.add_argument('--hash',required=True)
    consumer=sub.add_parser('consume');consumer.add_argument('--input',required=True);consumer.add_argument('--hash',required=True);consumer.add_argument('--destination',default='.');consumer.add_argument('--pool',action='store_true')
    gate=sub.add_parser('gate');gate.add_argument('--mode',required=True);gate.add_argument('--outcomes',required=True)
    args=parser.parse_args()
    if args.stage=='gate':print(json.dumps(check_execution(args.mode,json.loads(args.outcomes))));return
    source=evidence.source_reader(os.environ['GITHUB_SHA'])
    check(chain.impact.git('rev-parse','HEAD').decode('ascii').strip()==source['sha'],'Actual checkout differs')
    if args.stage=='produce':
        check(os.environ.get('GITHUB_ACTIONS')=='true','Producer runs only in Actions')
        captured=json.loads(Path(args.products).read_text(encoding='utf-8'));manifest,bodies=create_bundle(source,captured,'.',os.environ)
        output=Path(args.output);output.mkdir(parents=True,exist_ok=False)
        (output/'bundle.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
        for member,body in bodies.items():file=output/member;file.parent.mkdir(parents=True,exist_ok=True);file.write_bytes(body)
        print(json.dumps({'kind':manifest['kind'],'products':len(bodies)}));return
    if args.stage=='restore':
        check(os.environ.get('GITHUB_ACTIONS')=='true','Restore runs only in Actions')
        transport=evidence.transport_module.GitHubReadOnly(os.environ['GITHUB_REPOSITORY'],os.environ['GH_TOKEN'],allow_archive=True)
        print(json.dumps(restore_materialization(transport,source,os.environ,args.hash,args.output)));return
    if args.stage=='consume':
        receipt=consume(args.input,args.hash,source,os.environ,args.destination,args.pool)
        check(re.fullmatch('[a-f0-9]{40}',receipt['originBuildSHA']) and type(receipt['originArtifactId']) is int and receipt['originArtifactId']>0,'Original producer identity missing')
        if os.environ.get('GITHUB_ENV'):
            with Path(os.environ['GITHUB_ENV']).open('a',encoding='utf-8') as stream:
                stream.write('FARM_CI_NATIVE_ORIGIN_SHA='+receipt['originBuildSHA']+'\nFARM_CI_NATIVE_ARTIFACT_ID='+str(receipt['originArtifactId'])+'\n')
        print(json.dumps({'action':'reused-ci-bytes','originBuildSHA':receipt['originBuildSHA'],'consumerSHA':source['sha']}));return
    result='build';commitment='';reason='unsupported-event-or-no-proof'
    try:
        check(os.environ.get('GITHUB_EVENT_NAME') in ['pull_request','push'],'Only PR/main leaf reuse is enabled')
        event=json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text(encoding='utf-8'))
        if os.environ['GITHUB_EVENT_NAME']=='pull_request':
            check(event['pull_request']['head']['repo']['full_name']==os.environ['GITHUB_REPOSITORY'],'Fork PR reuses no bytes')
        else:check(os.environ.get('GITHUB_REF')=='refs/heads/main','Only main push reuse is enabled')
        transport=evidence.transport_module.GitHubReadOnly(os.environ['GITHUB_REPOSITORY'],os.environ['GH_TOKEN'],allow_archive=True)
        trusted=evidence.source_reader(transport.get('/branches/main')['commit']['sha'])
        decision,receipt,bodies=schedule(transport,source,trusted,os.environ)
        if receipt is not None:
            materialize(receipt,bodies,args.output)
            result='reuse';commitment=hashlib.sha256((Path(args.output)/'consumer.json').read_bytes()).hexdigest();reason='complete-trusted-ci-byte-proof'
    except (ValueError,KeyError,OSError,PermissionError,subprocess.SubprocessError,UnicodeError,zipfile.BadZipFile):
        # Missing evidence always builds. Never expose arbitrary exception text.
        reason='evidence-unavailable-build'
    decision={'kind':'actual-ci-native-execution-plan','chain':'ci','profile':'ci','sourceSHA':source['sha'],'sourceTree':source['tree'],
              'runId':int(os.environ['GITHUB_RUN_ID']),'attempt':int(os.environ['GITHUB_RUN_ATTEMPT']),'nativeMode':result,
              'nativeBuildMatrix':[abi for abi in chain.impact.TARGETS if result!='reuse' or abi!=ABI],
              'tsMatrix':list(chain.impact.PRIMARY),'examplesMatrix':list(chain.impact.PRIMARY),'resolutionCommitment':commitment or None,
              'reason':reason,'acceptanceInherited':False}
    decision['planHash']=chain.digest(decision)
    with Path(args.plan_output).open('x',encoding='utf-8') as stream:stream.write(json.dumps(decision,indent=2)+'\n')
    if os.environ.get('GITHUB_OUTPUT'):
        with Path(os.environ['GITHUB_OUTPUT']).open('a',encoding='utf-8') as stream:stream.write('mode='+result+'\nresolution_hash='+commitment+'\n')
    print(json.dumps({'nativeMode':result,'reason':reason,'acceptanceInherited':False}))


if __name__=='__main__':main()
