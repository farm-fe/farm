// Exercises shell routing and assembly with mock payloads; never compiles or publishes.
// cspell:words esac
const fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const temporaryRoot=require('node:os').tmpdir();
const fixture=fs.mkdtempSync(path.join(temporaryRoot,'farm-native-flow-'));
const unix=p=>process.platform==='win32'?'/'+p[0].toLowerCase()+p.slice(2).replaceAll('\\','/'):p;
const bash=process.platform==='win32'?'C:/Program Files/Git/bin/bash.exe':'bash';
const abis=['linux-x64-gnu','linux-x64-musl','darwin-x64','win32-x64-msvc','linux-arm64-musl','linux-arm64-gnu','darwin-arm64','win32-ia32-msvc','win32-arm64-msvc'];
const docs=['ssr-css','syntax-highlight','docs','docs-snapshot','docs-seo','docs-snippets'];
const supported=new Set(['win32-x64-msvc','linux-x64-gnu','linux-arm64-gnu','darwin-x64','darwin-arm64']);
const source=fs.readFileSync(path.join(root,'scripts/ci-build-rust-artifacts.sh'),'utf8');
const base=source.match(/^PLUGINS="([^"]+)"/m)[1].split(' ');
const plugins=[...base,...docs];
const pool=path.join(fixture,'artifacts'),trace=path.join(fixture,'trace.txt');
fs.mkdirSync(path.join(fixture,'scripts'),{recursive:true});
fs.copyFileSync(path.join(root,'scripts/check-native-build-node.mjs'),path.join(fixture,'scripts/check-native-build-node.mjs'));
fs.copyFileSync(path.join(root,'scripts/v2-node-engines.mjs'),path.join(fixture,'scripts/v2-node-engines.mjs'));
for(const dir of ['packages/core','packages/create-farm',...plugins.map(p=>'rust-plugins/'+p)])for(const abi of abis)fs.mkdirSync(path.join(fixture,dir,'npm',abi),{recursive:true});
const realNode=unix(process.execPath);
const stub=`set -eu
node() { test "$*" = "./scripts/check-native-build-node.mjs"; "${realNode}" "${unix(path.join(root,'scripts/check-native-build-node.mjs'))}"; }
cargo() { printf 'cargo|%s\\n' "$*" >> "$TRACE"; }
npm() {
  here="$(pwd)"
  case "$here" in
    */packages/core) suffix=""; file="farm.$FARM_BUILD_ABI.node" ;;
    */packages/create-farm) suffix="-create-farm"; file="create-farm.$FARM_BUILD_ABI.node" ;;
    */rust-plugins/*) plugin="$(basename "$here")"; suffix="-plugin-$plugin"; file="index.farm" ;;
    *) return 91 ;;
  esac
  destination="$ARTIFACT_POOL/$PROBE_SHA-$FARM_BUILD_ABI$suffix"
  mkdir -p "$destination"
  printf '%s|%s\\n' "$FARM_BUILD_ABI" "$suffix" > "$destination/$file"
  printf '%s|%s|%s\\n' "$FARM_BUILD_ABI" "$here" "$*" >> "$TRACE"
}
. "${unix(path.join(root,'scripts/ci-build-rust-artifacts.sh'))}"
`;
fs.writeFileSync(path.join(fixture,'build-probe.sh'),stub);
const sha='0123456789012345678901234567890123456789';
const env={...process.env,TRACE:unix(trace),ARTIFACT_POOL:unix(pool),PROBE_SHA:sha,FARM_BUILD_PROFILE:'ci'};
const receipts=[];
for(const abi of abis){cp.execFileSync(bash,['build-probe.sh'],{cwd:fixture,env:{...env,FARM_BUILD_ABI:abi},windowsHide:true,encoding:'utf8',timeout:30000});const names=fs.readdirSync(pool).filter(n=>n.startsWith(sha+'-'+abi+'-plugin-')).map(n=>n.slice((sha+'-'+abi+'-plugin-').length)).sort();const expected=[...base,...(supported.has(abi)?docs:[])].sort();assert.deepEqual(names,expected);receipts.push({abi,plugins:names.length,expected:names.length});}
const workflow=fs.readFileSync(path.join(root,'.github/workflows/release.yaml'),'utf8');
const block=workflow.match(/      - name: Move Artifacts\r?\n        run: \|\r?\n([\s\S]*?)\r?\n      - name: Install Dependencies/);
assert.ok(block,'move-artifacts block');
const assembly='set -eu\n'+block[1].split(/\r?\n/).map(l=>l.startsWith('          ')?l.slice(10):l).join('\n').replaceAll('${{ github.sha }}',sha).replaceAll('/tmp/artifacts/',unix(pool)+'/');
assert.ok(!assembly.includes('${{'));
fs.writeFileSync(path.join(fixture,'assemble.sh'),assembly);
cp.execFileSync(bash,['-n','assemble.sh'],{cwd:fixture,windowsHide:true});
cp.execFileSync(bash,['assemble.sh'],{cwd:fixture,windowsHide:true,encoding:'utf8',timeout:30000});
const moved=[];
for(const abi of abis){for(const plugin of [...base,...(supported.has(abi)?docs:[])]){const file=path.join(fixture,'rust-plugins',plugin,'npm',abi,'index.farm');assert.equal(fs.readFileSync(file,'utf8'),abi+'|-plugin-'+plugin+'\n');moved.push(file);}for(const [owner,binary]of [['core','farm'],['create-farm','create-farm']]){const file=path.join(fixture,'packages',owner,'npm',abi,binary+'.'+abi+'.node');assert.ok(fs.existsSync(file));moved.push(file);}}
for(const file of moved){const rel=path.relative(fixture,file).replaceAll('\\','/');const parts=rel.split('/');const abi=parts.at(-2),owner=parts[1],suffix=parts[0]==='rust-plugins'?'-plugin-'+owner:owner==='core'?'':'-create-farm';const dest=path.join(pool,sha+'-'+abi+suffix,path.basename(file));fs.mkdirSync(path.dirname(dest),{recursive:true});fs.copyFileSync(file,dest);}
fs.unlinkSync(path.join(pool,sha+'-linux-x64-gnu-plugin-docs-snippets','index.farm'));
const negative=cp.spawnSync(bash,['assemble.sh'],{cwd:fixture,windowsHide:true,encoding:'utf8',timeout:30000});
assert.notEqual(negative.status,0,'missing required Docs artifact must fail assembly');
assert.match(negative.stderr,/docs-snippets/);
const uploads=[];
for(const name of ['rust-build-user.yaml','rust-build-release.yaml']){const yaml=fs.readFileSync(path.join(root,'.github/workflows',name),'utf8').replace(/\r\n/g,'\n');for(const plugin of docs){const start=yaml.indexOf('      - name: Upload Plugin '+plugin+'\n');assert.ok(start>=0,plugin+' upload');const step=yaml.slice(start).split('\n      - name: ')[0];assert.ok(step.includes('if-no-files-found: error'));assert.ok(step.includes("contains(fromJSON('[\"win32-x64-msvc\",\"linux-x64-gnu\",\"linux-arm64-gnu\",\"darwin-x64\",\"darwin-arm64\"]'), matrix.settings.abi)"));assert.ok(step.includes('rust-plugins/'+plugin+'/npm/${{ matrix.settings.abi }}/index.farm'));}uploads.push({workflow:name,guardedDocsUploads:docs.length});}
const out={utc:new Date().toISOString(),status:'passed',fixture,receipts,movedArtifacts:moved.length,missingArtifactRejected:true,negativeExit:negative.status,negativeStderr:negative.stderr,uploads,scope:'Actual shell build-selection and release-assembly logic with fixture-only mocked npm/cargo/native payloads; no compilation, installation, registry writes or publication. Not native execution or GitHub Actions success.'};
// Keep all generated mock binaries outside the repository.
assert.equal(path.dirname(fixture),temporaryRoot);
assert.ok(path.basename(fixture).startsWith('farm-native-flow-'));
fs.rmSync(fixture,{recursive:true,force:true});
console.log(JSON.stringify({status:out.status,targets:receipts.length,movedArtifacts:moved.length,missingArtifactRejected:true,uploads}));
