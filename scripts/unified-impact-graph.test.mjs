// cspell:words cp1252 surrogateescape PYTHONIOENCODING
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {readGitSuiteFiles} from './ci-suite-inventory.mjs';
import {unifiedGraph} from './unified-impact-graph.mjs';

const root=fileURLToPath(new URL('../',import.meta.url));
const script=fileURLToPath(new URL('./unified-impact-graph.py',import.meta.url));
const actual=readGitSuiteFiles(root);
function project(input,encoding) {
 return spawnSync(process.env.FARM_GRAPH_PYTHON??'python3',['-B',script],{
  input:Buffer.isBuffer(input)?input:Buffer.from(JSON.stringify({snapshots:input}),'utf8'),
  encoding:'utf8',env:{...process.env,PYTHONIOENCODING:encoding},timeout:30000,maxBuffer:32*1024**2
 });
}
const expected=unifiedGraph([actual.files]);
for(const encoding of ['cp1252:surrogateescape','ascii:surrogateescape','utf-8']) {
 test(`real committed graph is identical with stdin code page ${encoding}`,()=>{
  const result=project([actual.files],encoding);
  assert.equal(result.status,0,result.stderr);
  assert.deepEqual(JSON.parse(result.stdout),expected);
 });
}
test('graph transport preserves Unicode owner paths, metadata and local dependency edges',()=>{
 const files={
  'packages/组件/package.json':JSON.stringify({name:'component',version:'1.0.0',description:'苏向夜 😀'}),
  'packages/app/package.json':JSON.stringify({name:'app',version:'1.0.0',dependencies:{component:'file:../组件'}})
 };
 const expected=unifiedGraph([files]);
 const result=project([files],'cp1252:surrogateescape');
 assert.equal(result.status,0,result.stderr);
 assert.deepEqual(JSON.parse(result.stdout),expected);
 assert.ok(expected.nodes['npm:packages/app'].preparationDeps.includes('npm:packages/组件'));
});
test('tracked binary assets remain outside text graph projection without being decoded or discarded from Git inventory',()=>{
 const binary=actual.entries.filter(row=>/\.(png|jpg|wasm|woff2?)$/.test(row.file));
 assert.ok(binary.length>0);
 for(const row of binary)assert.equal(Object.hasOwn(actual.files,row.file),false,row.file);
 assert.ok(Object.values(actual.files).some(text=>/[^\x00-\x7f]/.test(text)));
});
test('invalid UTF-8 JSON bytes fail closed rather than dropping or replacing input',()=>{
 const input=Buffer.concat([Buffer.from('{"snapshots":[{"packages/app/package.json":"'),Buffer.from([0x90]),Buffer.from('"}]}')]);
 const result=project(input,'cp1252:surrogateescape');
 assert.notEqual(result.status,0);assert.match(result.stderr,/UnicodeDecodeError/);
 assert.equal(result.stdout,'');
});
test('escaped unpaired surrogate input also fails closed',()=>{
 const result=project([{'packages/app/package.json':'\udc90'}],'utf-8');
 assert.notEqual(result.status,0);assert.match(result.stderr,/UnicodeEncodeError/);
 assert.equal(result.stdout,'');
});
test('strict protocol reads bytes and does not depend on process stdin locale',()=>{
 const source=fs.readFileSync(script,'utf8');
 assert.match(source,/sys\.stdin\.buffer\.read\(\)\.decode\("utf-8"\)/);
 assert.doesNotMatch(source,/errors\s*=\s*["'](?:ignore|replace)["']/);
});
