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
    pr=event['pull_request'];repo=env['GITHUB_REPOSITORY'];sha=pr['merge_commit_sha']
    check(env['GITHUB_EVENT_NAME']=='pull_request' and event['action']=='closed' and pr['merged'] is True,'Only a merged PR authorizes Cargo source release')
    check(pr['base']['ref']=='main' and pr['base']['repo']['full_name']==repo and pr['head']['repo']['full_name']==repo,'Wrong base or fork Cargo release')
    check(event['number']==2329 or pr['head']['ref'].startswith('changeset-release/'),'Cargo trigger exceeds original release branches')
    check(re.fullmatch('[a-f0-9]{40}',sha) and git('git','rev-parse','HEAD')==sha,'Cargo source is not exact merge commit')
    tree=git('git','rev-parse','HEAD^{tree}')
    check(tree==git('git','rev-parse',pr['head']['sha']+'^{tree}'),'Reviewed head and merge tree differ')
    return {'repository':repo,'repositoryId':int(env['GITHUB_REPOSITORY_ID']),'runId':int(env['GITHUB_RUN_ID']),
            'sourceSHA':sha,'sourceTree':tree,'pr':event['number'],'headSHA':pr['head']['sha'],'baseSHA':pr['base']['sha'],'workflow':WORKFLOW}

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

def restore(directory,expected):
    name='cargo-source-'+str(expected['runId']);base='repos/'+expected['repository'];rows=api(base+'/actions/runs/'+str(expected['runId'])+'/artifacts?per_page=100')['artifacts']
    matches=[row for row in rows if row['name']==name];check(len(matches)<=1,'Ambiguous Cargo frozen artifact')
    if not matches:
        check(int(os.environ['GITHUB_RUN_ATTEMPT'])==1,'Original Cargo archive missing on retry; possible partial publication cannot be rebuilt')
        return False
    row=matches[0];check(not row['expired'],'Original Cargo archive expired; refusing to rebuild a possible partial publication')
    check(row['workflow_run']['id']==expected['runId'] and 0<row['size_in_bytes']<512*1024**2,'Cargo artifact origin or size differs')
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
    check(producer_run['path']==WORKFLOW and producer_run['event']=='pull_request' and producer_run['head_sha']==expected['sourceSHA'] and producer_run['repository']['id']==expected['repositoryId'] and producer_run['head_repository']['id']==expected['repositoryId'],'Cargo producer run identity differs')
    jobs=api(base+f'/actions/runs/{expected["runId"]}/attempts/{attempt}/jobs?per_page=100')['jobs']
    producers=[job for job in jobs if job['name']=='Prepare Cargo source archives'];check(len(producers)==1,'Cargo producer job absent')
    check(producers[0]['status']=='completed' and producers[0]['conclusion']=='success','Cargo producer job did not succeed')
    for name in ['Verify reviewed version provenance and semantic changes','Package and verify exact source crates']:
        matching=[step for step in producers[0]['steps'] if step['name']==name]
        check(len(matching)==1 and matching[0]['conclusion']=='success','Cargo source verification step missing or failed')
    steps=[step for step in producers[0]['steps'] if step['name']=='Freeze verified Cargo source archives']
    check(len(steps)==1 and steps[0]['conclusion']=='success' and steps[0]['started_at']<=row['created_at']<=steps[0]['completed_at'],'Cargo artifact upload was not a completed producer step')
    return True

def prepare(directory,expected):
    proof=json.loads(Path('cargo-version-proof.json').read_text());check(proof['mode']=='verified-version-only' and proof['headSHA']==expected['headSHA'] and proof['baseSHA']==expected['baseSHA'],'Cargo version provenance is incomplete')
    evidence=load('cargo_source_evidence','incremental-chain-evidence.py')
    before=evidence.source_reader(expected['baseSHA']);source=evidence.source_reader(expected['sourceSHA'])
    selected=select_packages(before['files'],source['files'])
    check(run('git','diff','--name-only','HEAD')=='','Cargo preparation requires a clean tracked source')
    directory.mkdir();(directory/'files').mkdir()
    metadata=json.loads(run('cargo','metadata','--locked','--offline','--format-version','1','--no-deps'))
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
    receipt={'schemaVersion':1,'identity':expected,'producerAttempt':int(os.environ['GITHUB_RUN_ATTEMPT']),'toolchain':run('rustc','-vV'),'verified':True,'selection':selected,'selectionHash':digest(encoded(selected)),'records':records}
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
        restored=restore(directory,expected)
        with open(os.environ['GITHUB_OUTPUT'],'a') as stream:stream.write('restored='+str(restored).lower()+'\n')
    elif stage=='prepare':prepare(directory,expected)
    else:
        check(os.environ.get('CARGO_REGISTRY_TOKEN'),'Existing registry secret required')
        check(restore(directory,expected),'Publication requires frozen original producer artifact');publish(directory,expected)
if __name__=='__main__':main()
