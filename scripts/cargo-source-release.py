"""Cargo source publication, independent of npm/native incremental release."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.request
import zipfile

ROOT=Path(__file__).resolve().parent.parent
WORKFLOW='.github/workflows/cargo-source-release.yaml'
def check(value,message):
    if not value: raise ValueError(message)
def digest(value): return hashlib.sha256(value).hexdigest()
def encoded(value): return json.dumps(value,sort_keys=True,separators=(',',':')).encode()
def run(*args,cwd=ROOT): return subprocess.check_output(args,cwd=cwd,text=True).strip()
def api(route): return json.loads(run('gh','api',route))
def load(name,file):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/file);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def identity(event,env,git=run):
    if env.get('GITHUB_EVENT_NAME')=='push':
        sha=env['GITHUB_SHA'];repo=env['GITHUB_REPOSITORY']
        check(env.get('GITHUB_REF')=='refs/heads/main' and event.get('ref')=='refs/heads/main' and event.get('after')==sha and not event.get('forced') and not event.get('deleted'),'Cargo requires fixed main push')
        check(env.get('GITHUB_WORKFLOW_SHA')==sha and env.get('GITHUB_WORKFLOW_REF')==repo+'/'+WORKFLOW+'@refs/heads/main','Cargo workflow/source identity differs')
        check(re.fullmatch('[a-f0-9]{40}',sha) and git('git','rev-parse','HEAD')==sha,'Cargo checkout differs')
        return {'repository':repo,'repositoryId':int(env['GITHUB_REPOSITORY_ID']),'runId':int(env['GITHUB_RUN_ID']),
                'sourceSHA':sha,'sourceTree':git('git','rev-parse','HEAD^{tree}'),'headSHA':sha,'event':'push','workflow':WORKFLOW}
    pr=event['pull_request'];repo=env['GITHUB_REPOSITORY'];sha=pr['merge_commit_sha']
    check(env['GITHUB_EVENT_NAME']=='pull_request' and event['action']=='closed' and pr['merged'] is True,'Only a merged PR authorizes Cargo source release')
    check(pr['base']['ref']=='main' and pr['base']['repo']['full_name']==repo and pr['head']['repo']['full_name']==repo,'Wrong base or fork Cargo release')
    check(event['number'] in [2329,2330] or pr['head']['ref'].startswith('changeset-release/'),'Cargo trigger exceeds original release branches')
    check(re.fullmatch('[a-f0-9]{40}',sha) and git('git','rev-parse','HEAD')==sha,'Cargo source is not exact merge commit')
    tree=git('git','rev-parse','HEAD^{tree}')
    head_tree=git('git','rev-parse',pr['head']['sha']+'^{tree}')
    if tree != head_tree:
        parent=git('git','rev-parse',sha+'^1')
        check(tree==git('git','merge-tree','--write-tree',parent,pr['head']['sha']).splitlines()[0],'Reviewed change differs from actual merge tree')
    return {'repository':repo,'repositoryId':int(env['GITHUB_REPOSITORY_ID']),'runId':int(env['GITHUB_RUN_ID']),
            'sourceSHA':sha,'sourceTree':tree,'pr':event['number'],'headSHA':pr['head']['sha'],'eventBaseSHA':pr['base']['sha'],'workflow':WORKFLOW}

def select_prepared_packages(files, plan, retained, lookup):
    """First prepared family: no base/head version delta and no registry writes.

    Existing versions may only be retained by an explicitly reviewed archive
    checksum. Every other member is packaged and must pass the normal complete
    immutable-byte preflight before any upload, including race/retry cases.
    """
    import tomllib
    declared = plan['cargo']
    check(len({row['name'] for row in declared}) == len(declared), 'Duplicate prepared Cargo member')
    allowed = {row['name']: row for row in declared}
    check(len({row['name'] for row in retained}) == len(retained), 'Duplicate retained Cargo member')
    for row in declared:
        manifest = tomllib.loads(files[row['manifestPath']].decode('utf-8'))['package']
        check(manifest['name'] == row['name'] and manifest['version'] == row['version'], 'Prepared Cargo source version differs')
        check(re.fullmatch(r'\d+\.\d+\.\d+', row['version']), 'Prepared Cargo stable version required')
        check(manifest.get('publish', True) is not False, 'Private Cargo package in prepared family')
    kept = set()
    for row in retained:
        check(row['name'] in allowed and allowed[row['name']]['version'] == row['version'], 'Retained crate is outside prepared family')
        check(re.fullmatch('[a-f0-9]{64}', row['sha256']), 'Invalid retained Cargo archive checksum')
        check(lookup(row) == row['sha256'], 'Retained Cargo archive is missing, yanked or has different bytes')
        kept.add(row['name'])
    return [row for row in declared if row['name'] not in kept]

def public_source_seeds(metadata, selected):
    """Find registry twins hidden by local paths in the selected locked closure.

    Use the all-features, all-target Cargo graph so optional, build and dev
    dependencies are covered too. Selected unpublished crates stay local: Cargo's
    multi-package staging registry supplies those during offline packaging.
    """
    packages = {row['id']: row for row in metadata['packages']}
    nodes = {row['id']: row for row in metadata['resolve']['nodes']}
    selected_ids = set()
    for row in selected:
        matches = [pkg['id'] for pkg in packages.values()
                   if pkg['name'] == row['name'] and pkg['version'] == row['version'] and pkg['source'] is None]
        check(len(matches) == 1, 'Missing exact selected Cargo source')
        selected_ids.add(matches[0])
    pending = list(selected_ids); seen = set(); seeds = {}
    while pending:
        package_id = pending.pop()
        if package_id in seen: continue
        seen.add(package_id)
        check(package_id in nodes and package_id in packages, 'Incomplete locked Cargo dependency graph')
        pkg = packages[package_id]; node = nodes[package_id]
        pending.extend(node['dependencies'])
        if pkg['source'] is None and package_id not in selected_ids:
            key = (pkg['name'], pkg['version'])
            check(key not in seeds, 'Ambiguous local Cargo registry twin')
            seeds[key] = {'name': pkg['name'], 'version': pkg['version'], 'features': sorted(node['features'])}
    return [seeds[key] for key in sorted(seeds)]


def fetch_public_sources(seeds):
    # Resolve each exact registry twin outside the repository, without workspace
    # patches or repository .cargo configuration. Independent roots also allow
    # multiple versions of the same crate without artificial feature unification.
    if not seeds: return
    toolchain = run('rustup', 'show', 'active-toolchain').split()[0]
    env = {**os.environ, 'RUSTUP_TOOLCHAIN': toolchain}
    with tempfile.TemporaryDirectory(prefix='farm-public-cargo-') as temporary:
        for index, row in enumerate(seeds):
            root = Path(temporary) / str(index); (root / 'src').mkdir(parents=True)
            (root / 'src/lib.rs').write_text('')
            dependency = ('seed = { package = ' + json.dumps(row['name']) + ', version = ' +
                          json.dumps('=' + row['version']) + ', registry = "crates-io", default-features = false, features = ' +
                          json.dumps(row['features']) + ' }')
            (root / 'Cargo.toml').write_text('[package]\nname="farm-public-source-validation"\nversion="0.0.0"\nedition="2024"\npublish=false\n[workspace]\n[dependencies]\n' + dependency + '\n')
            print('Preparing public Cargo source ' + row['name'] + '@' + row['version'], flush=True)
            # Only this disposable manifest gets a new lockfile. The checked-in
            # lock and the final --locked --offline package command never change.
            subprocess.run(['cargo', 'generate-lockfile'], cwd=root, env=env, check=True)
            subprocess.run(['cargo', 'fetch', '--locked'], cwd=root, env=env, check=True)

def select_packages(before,after):
    chain=load('cargo_source_chain','incremental-chain-plan.py')
    old_nodes,old_issues=chain.repository_graph(before);nodes,issues=chain.repository_graph(after)
    check(not old_issues and not issues,'Cargo source graph unresolved')
    old={k:v for k,v in chain.independent_family(before,old_nodes).items() if v['kind']=='cargo'}
    current={k:v for k,v in chain.independent_family(after,nodes).items() if v['kind']=='cargo'}
    check(set(old)==set(current),'Cargo source family changes require separate review')
    selected=[]
    for key,row in sorted(current.items()):
        if row['version']==old[key]['version']: continue
        versions=[]
        for value in [old[key]['version'],row['version']]:
            check(re.fullmatch(r'\d+\.\d+\.\d+',value),'Cargo stable version required');versions.append(tuple(map(int,value.split('.'))))
        check(versions[1]>versions[0],'Cargo version must advance');selected.append(row)
    return selected

def safe_archive(body,name,version):
    prefix=name+'-'+version+'/'
    check(0<len(body)<100*1024**2,'Cargo archive size exceeds bound')
    files={};total=0
    with tarfile.open(fileobj=io.BytesIO(body),mode='r:gz') as archive:
        members=archive.getmembers();check(len(members)<=20000,'Too many Cargo archive members')
        for item in members:
            check(item.name.startswith(prefix) and '\\' not in item.name and '..' not in Path(item.name).parts,'Cargo archive path escapes package')
            if item.isdir():continue
            check(item.isfile() and item.name not in files,'Cargo archive link or duplicate member')
            total+=item.size;check(0<=item.size<=100*1024**2 and total<=200*1024**2,'Cargo expanded archive exceeds bound')
            files[item.name]=archive.extractfile(item).read()
    import tomllib
    manifest=tomllib.loads(files[prefix+'Cargo.toml'].decode())
    check(manifest['package']['name']==name and manifest['package']['version']==version,'Normalized Cargo identity differs')
    return files

def bind_normalized_metadata(files,pkg,normalized):
    prefix=pkg['name']+'-'+pkg['version']+'/'
    check(Path(normalized).read_bytes()==files[prefix+'Cargo.toml'],'Extracted Cargo manifest differs from frozen archive')
    readme=pkg.get('readme')
    if readme:
        check(not Path(readme).is_absolute() and '..' not in Path(readme).parts and '\\' not in readme,'Cargo README path escapes archive')
        check(prefix+readme in files and (Path(normalized).parent/readme).read_bytes()==files[prefix+readme],'Extracted README differs from frozen archive')
    return files.get(prefix+readme) if readme else None

def verify_frozen(directory,expected):
    directory=Path(directory);receipt=json.loads((directory/'receipt.json').read_bytes())
    check(receipt['identity']==expected and receipt['verified'] is True,'Frozen Cargo source identity differs')
    check(receipt['selectionHash']==digest(encoded(receipt['selection'])),'Cargo selection hash differs')
    check(len(receipt['records'])==len(receipt['selection']),'Cargo inventory incomplete')
    selected={row['name']:row['version'] for row in receipt['selection']};seen=set()
    retained=receipt.get('retained',[])
    if receipt.get('preparedFamily') is not None:
        family=receipt['preparedFamily']
        check(receipt['preparedFamilyHash']==digest(encoded(family)), 'Prepared Cargo family hash differs')
        check(len({row['name'] for row in family})==len(family), 'Duplicate prepared Cargo member')
        check(len({row['name'] for row in retained})==len(retained), 'Duplicate retained Cargo member')
        inventory={row['name']:row['version'] for row in family}
        check(not (set(selected)&{row['name'] for row in retained}), 'Retained Cargo version cannot be republished')
        check(inventory=={**selected,**{row['name']:row['version'] for row in retained}}, 'Prepared Cargo inventory incomplete')
        check(receipt['retainedHash']==digest(encoded(retained)), 'Retained Cargo receipt hash differs')
        for row in retained:check(re.fullmatch('[a-f0-9]{64}',row['sha256']), 'Invalid retained archive checksum')
    else:check(not retained,'Retained Cargo versions require prepared family proof')
    for row in receipt['records']:
        check(row['name'] not in seen and selected.get(row['name'])==row['version'],'Extra or duplicate frozen Cargo package')
        check(set(row['dependencies'])<=seen,'Cargo dependency order differs');seen.add(row['name'])
        for key in ['archive','metadata']:
            name=row[key];check(Path(name).name==name and name not in ['.','..'],'Cargo receipt path escapes')
            file=directory/'files'/name;check(file.is_file() and not file.is_symlink(),'Frozen Cargo member missing or redirected')
            body=file.read_bytes();check(digest(body)==row[key+'Sha256'] and len(body)==row[key+'Bytes'],'Frozen Cargo bytes differ')
        safe_archive((directory/'files'/row['archive']).read_bytes(),row['name'],row['version'])
        meta=json.loads((directory/'files'/row['metadata']).read_bytes());check(meta['name']==row['name'] and meta['vers']==row['version'],'Upload metadata identity differs')
    return receipt

def restore(directory,expected,*,proof_provider=None,binding_output=None):
    name='cargo-source-'+str(expected['runId']);base='repos/'+expected['repository'];rows=api(base+'/actions/runs/'+str(expected['runId'])+'/artifacts?per_page=100')['artifacts']
    check(len(rows)<100,'Cargo frozen artifact listing exceeds verification bound')
    matches=[row for row in rows if row['name']==name];check(len(matches)<=1,'Ambiguous Cargo frozen artifact')
    if not matches:
        attempt=int(os.environ['GITHUB_RUN_ATTEMPT'])
        if attempt>1:
            retry=load('cargo_retry_safety','release-retry-safety.py')
            proof=retry.verify_unfrozen_retry(lambda route:api(base+route), run_id=expected['runId'], source_sha=expected['sourceSHA'],
                current_attempt=attempt, workflow=WORKFLOW, producer='Prepare Cargo source archives',
                freeze_step='Freeze verified Cargo source archives', publisher='Publish frozen Cargo source archives', events=[expected.get('event','pull_request')])
            print(json.dumps({'unfrozenRetryProof':proof}),flush=True)
        return False
    row=matches[0];check(not row['expired'],'Original Cargo archive expired; refusing to rebuild a possible partial publication')
    check(row['workflow_run']['id']==expected['runId'] and 0<row['size_in_bytes']<512*1024**2,'Cargo artifact origin or size differs')
    provenance=load('cargo_frozen_provenance','release-provenance.py')
    class ReadOnly:
        def get(self,route):return api(base+route)
        def archive(self,identity):return subprocess.check_output(['gh','api',base+'/actions/artifacts/'+str(identity)+'/zip'],timeout=120)
    signed_meta,origin,origin_job=provenance.find_frozen_binding(ReadOnly(),
        {'sha':expected['sourceSHA'],'tree':expected['sourceTree']},'cargo',expected['runId'],
        int(os.environ['GITHUB_RUN_ATTEMPT']),row['id'],'cargo',proof_provider=proof_provider)
    check(all(signed_meta.get(key)==row.get(key) for key in ['id','name','digest','size_in_bytes']), 'Cargo discovered archive differs from signed binding')
    body=subprocess.check_output(['gh','api',base+'/actions/artifacts/'+str(row['id'])+'/zip'],timeout=120)
    check(len(body)==row['size_in_bytes'] and row['digest']=='sha256:'+digest(body),'Cargo artifact digest differs')
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        check(sum(member.file_size for member in archive.infolist())<=512*1024**2,'Cargo artifact expanded total exceeds bound')
        names=archive.namelist();check(len(names)==len(set(names)) and len(names)<500,'Cargo artifact inventory invalid')
        for member in archive.infolist():
            check(member.filename=='receipt.json' or re.fullmatch(r'files/[A-Za-z0-9_.-]+',member.filename),'Unexpected Cargo artifact member')
            check(member.file_size<100*1024**2,'Frozen artifact member too large')
        directory.mkdir(exist_ok=False)
        for member in archive.infolist():
            destination=directory/member.filename;destination.parent.mkdir(exist_ok=True);destination.write_bytes(archive.read(member))
    receipt=verify_frozen(directory,expected)
    attempt=receipt['producerAttempt'];check(0<attempt<=int(os.environ['GITHUB_RUN_ATTEMPT']),'Invalid Cargo producer attempt')
    producer_run=api(base+f'/actions/runs/{expected["runId"]}/attempts/{attempt}')
    check(producer_run['path']==WORKFLOW and producer_run['event']==expected.get('event','pull_request') and producer_run['head_sha']==expected['sourceSHA'] and producer_run['repository']['id']==expected['repositoryId'] and producer_run['head_repository']['id']==expected['repositoryId'],'Cargo producer run identity differs')
    jobs=api(base+f'/actions/runs/{expected["runId"]}/attempts/{attempt}/jobs?per_page=100')['jobs']
    producers=[job for job in jobs if job['name']=='Prepare Cargo source archives'];check(len(producers)==1,'Cargo producer job absent')
    check(producers[0]['status']=='completed' and producers[0]['conclusion']=='success','Cargo producer job did not succeed')
    for name in [('Verify fixed main versions and Changesets readiness' if expected.get('event')=='push' else 'Verify reviewed version provenance and semantic changes'),'Package and verify exact source crates']:
        matching=[step for step in producers[0]['steps'] if step['name']==name]
        check(len(matching)==1 and matching[0]['conclusion']=='success','Cargo source verification step missing or failed')
    steps=[step for step in producers[0]['steps'] if step['name']=='Freeze verified Cargo source archives']
    check(len(steps)==1 and steps[0]['status']=='completed' and steps[0]['conclusion']=='success'
          and origin['run_attempt']==attempt and origin_job['id']==producers[0]['id'], 'Cargo artifact signed producer differs')
    if binding_output is not None:
        binding_output.update(artifact_id=str(row['id']),artifact_digest=row['digest'].removeprefix('sha256:'),producer_attempt=str(attempt))
    return True

def prepare(directory,expected):
    proof=json.loads(Path('cargo-version-proof.json').read_text())
    registry_snapshot=None
    if proof.get('mode')=='verified-main-registry-selection':
        check(proof.get('releaseReady') is True and proof['sourceSHA']==expected['sourceSHA'] and proof['sourceTree']==expected['sourceTree'],'Cargo source readiness differs')
        evidence=load('cargo_registry_source','incremental-chain-evidence.py');source=evidence.source_reader(expected['sourceSHA'])
        selector=load('cargo_registry_selection','release-registry-selection.py')
        registry_snapshot=selector.collect(source,'cargo',selector.Registry().lookup)
        declared=selector.validate(source,'cargo',registry_snapshot)
        family=list(declared.values());selected=[row for key,row in declared.items() if registry_snapshot['packages'][key]['status']=='missing']
        retained=[{'name':row['name'],'version':row['version'],'sha256':registry_snapshot['packages'][key]['archiveSha256']} for key,row in declared.items() if registry_snapshot['packages'][key]['status']=='published']
    else:
        check(proof['mode'] in ['verified-version-only','verified-prepared-migration'] and proof['headSHA']==expected['headSHA'] and re.fullmatch('[a-f0-9]{40}',proof['baseSHA']) and re.fullmatch('[a-f0-9]{64}',proof['receiptHash']),'Cargo version provenance is incomplete')
        evidence=load('cargo_source_evidence','incremental-chain-evidence.py')
        before=evidence.source_reader(proof['baseSHA']);source=evidence.source_reader(expected['sourceSHA'])
        retained=[];family=None
        if proof['mode']=='verified-prepared-migration':
            check(expected['pr']==2330 and proof['sourceSHA']==expected['sourceSHA'] and proof['sourceTree']==expected['sourceTree'], 'Prepared Cargo proof belongs to another source')
            policy=json.loads(source['files']['docs/releases/prepared-release-2330.json'])
            check(digest(json.dumps(policy,separators=(',',':'),ensure_ascii=False).encode())==proof['policyHash'], 'Prepared policy hash differs')
            check(policy['retainedCrates']==proof['retainedCrates'], 'Retained Cargo proof differs from reviewed policy')
            plan=json.loads(source['files'][policy['candidate']]);retained=policy['retainedCrates'];family=plan['cargo']
            selected=select_prepared_packages(source['files'],plan,retained,Registry().lookup)
        else:selected=select_packages(before['files'],source['files'])
    check(run('git','diff','--name-only','HEAD')=='','Cargo preparation requires a clean tracked source')
    directory.mkdir();(directory/'files').mkdir()
    metadata=json.loads(run('cargo','metadata','--locked','--offline','--format-version','1','--no-deps'))
    if selected:
        resolved=json.loads(run('cargo','metadata','--locked','--offline','--all-features','--format-version','1'))
        fetch_public_sources(public_source_seeds(resolved,selected))
    target=directory.resolve()/'target';env={**os.environ,'CARGO_TARGET_DIR':str(target)}
    if selected: subprocess.run(['cargo','package','--locked','--offline','--registry','crates-io',*sum((['-p',row['name']] for row in selected),[])],cwd=ROOT,env=env,check=True)
    order=json.loads(subprocess.check_output(['node','--input-type=module','-e',"import {rustPublicationOrder} from './scripts/release-rust-metadata.mjs';let s='';for await(const c of process.stdin)s+=c;const v=JSON.parse(s);console.log(JSON.stringify(rustPublicationOrder({cargo:v.selection},v.metadata,{includeUploadMetadata:false})));"],input=json.dumps({'selection':selected,'metadata':metadata}).encode(),cwd=ROOT))
    records=[]
    for row in order:
        name,version=row['name'],row['version'];filename=name+'-'+version+'.crate';body=(target/'package'/filename).read_bytes();archive_files=safe_archive(body,name,version)
        normalized=target/'package'/(name+'-'+version)/'Cargo.toml'
        meta=json.loads(run('cargo','metadata','--offline','--format-version','1','--no-deps','--manifest-path',str(normalized)))
        packages=[pkg for pkg in meta['packages'] if pkg['name']==name and pkg['version']==version];check(len(packages)==1,'Verified normalized Cargo metadata missing')
        pkg=packages[0];check(pkg['source'] is None and Path(pkg['manifest_path']).resolve()==normalized.resolve(),'Normalized root metadata differs')
        archive_readme=bind_normalized_metadata(archive_files,pkg,normalized)
        pkg['source']='registry+https://github.com/rust-lang/crates.io-index'
        upload=subprocess.check_output(['node','--input-type=module','-e',"import {uploadMetadata} from './scripts/release-rust-metadata.mjs';let s='';for await(const c of process.stdin)s+=c;console.log(JSON.stringify(uploadMetadata(JSON.parse(s))));"],input=json.dumps(pkg).encode(),cwd=ROOT)
        check(json.loads(upload).get('readme')==(archive_readme.decode() if archive_readme is not None else None),'Upload README differs from frozen archive')
        (directory/'files'/filename).write_bytes(body);meta_name=name+'-'+version+'.json';(directory/'files'/meta_name).write_bytes(upload)
        records.append({**row,'archive':filename,'archiveSha256':digest(body),'archiveBytes':len(body),'metadata':meta_name,'metadataSha256':digest(upload),'metadataBytes':len(upload)})
    check(run('git','diff','--name-only','HEAD')=='','Cargo package altered tracked source')
    receipt={'schemaVersion':1,'identity':expected,'producerAttempt':int(os.environ['GITHUB_RUN_ATTEMPT']),'toolchain':run('rustc','-vV'),'verified':True,'versionBaseSHA':proof['baseSHA'],'versionProofHash':proof['receiptHash'],'packageCommand':['cargo','package','--locked','--offline','--registry','crates-io',*sum((['-p',row['name']] for row in selected),[])],'selection':selected,'selectionHash':digest(encoded(selected)),'records':records}
    if registry_snapshot is not None:receipt['registrySnapshot']=registry_snapshot
    if family is not None:receipt.update(preparedFamily=family,preparedFamilyHash=digest(encoded(family)),retained=retained,retainedHash=digest(encoded(retained)))
    (directory/'receipt.json').write_bytes(encoded(receipt));verify_frozen(directory,expected)

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        raise ValueError('Cargo registry redirects are not authorized')

class Registry:
    def request(self,url,body=None):
        check(url.startswith('https://crates.io/api/v1/crates/'),'Unexpected registry route')
        headers={'User-Agent':'farm-cargo-source-release','Accept':'application/json'}
        if body is not None:headers.update({'Authorization':os.environ['CARGO_REGISTRY_TOKEN'],'Content-Type':'application/octet-stream'})
        try:
            opener=urllib.request.build_opener(NoRedirect())
            with opener.open(urllib.request.Request(url,data=body,headers=headers,method='PUT' if body is not None else 'GET'),timeout=30) as response:return json.load(response)
        except urllib.error.HTTPError as error:
            if error.code==404 and body is None:return None
            raise
    def lookup(self,row):
        value=self.request('https://crates.io/api/v1/crates/'+row['name']+'/'+row['version'])
        if value is None:return None
        check(value['version']['yanked'] is False,'Cargo version is yanked');return value['version']['checksum']
    def upload(self,metadata,archive):
        return self.request('https://crates.io/api/v1/crates/new',struct.pack('<I',len(metadata))+metadata+struct.pack('<I',len(archive))+archive)

def publish(directory,expected,registry=None,sleep=time.sleep):
    directory=Path(directory);receipt=verify_frozen(directory,expected);registry=registry or Registry()
    # Preflight the complete selection before the first side effect.
    for row in receipt.get('retained',[]):
        check(registry.lookup(row)==row['sha256'],'Retained Cargo archive changed or disappeared before publication')
    for row in receipt['records']:
        existing=registry.lookup(row);check(existing in [None,row['archiveSha256']],'Immutable Cargo version conflict')
    state={'identity':expected,'producerAttempt':receipt['producerAttempt'],'packages':[]}
    for row in receipt['records']:
        record={'name':row['name'],'version':row['version'],'status':'pending'};state['packages'].append(record)
        try:
            if registry.lookup(row) is None:registry.upload((directory/'files'/row['metadata']).read_bytes(),(directory/'files'/row['archive']).read_bytes())
            for attempt in range(6):
                if registry.lookup(row)==row['archiveSha256']:break
                check(attempt<5,'Cargo registry did not confirm frozen archive');sleep(3)
            record['status']='verified'
        except Exception:
            record['status']='failed';raise
        finally:(directory/'publication.json').write_bytes(encoded(state))
    return state

def main():
    stage=sys.argv[1];check(stage in ['restore','prepare','publish'],'Unknown Cargo stage')
    event=json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text());expected=identity(event,os.environ)
    directory=ROOT/'cargo-source-frozen'
    if stage=='restore':
        original_binding={}
        restored=restore(directory,expected,binding_output=original_binding)
        with open(os.environ['GITHUB_OUTPUT'],'a') as stream:
            stream.write('restored='+str(restored).lower()+'\n')
            if restored:
                stream.write('publish_required='+str(bool(verify_frozen(directory,expected)['selection'])).lower()+'\n')
                for key,value in original_binding.items():stream.write(key+'='+value+'\n')
    elif stage=='prepare':
        prepare(directory,expected)
        with open(os.environ['GITHUB_OUTPUT'],'a') as stream:stream.write('publish_required='+str(bool(verify_frozen(directory,expected)['selection'])).lower()+'\n')
    else:
        check(os.environ.get('CARGO_REGISTRY_TOKEN'),'Existing registry secret required')
        check(restore(directory,expected),'Publication requires frozen original producer artifact');publish(directory,expected)
if __name__=='__main__':main()
