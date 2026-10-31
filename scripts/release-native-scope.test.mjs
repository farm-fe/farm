import test from 'node:test';
import assert from 'node:assert/strict';
import { releaseNativeScope } from './release-native-scope.mjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {nativeArtifactInventory} from './native-artifact-inventory.mjs';
const base = { profile: 'release', plugins: ['react', 'vue'], names: { react: '@farmfe/plugin-react', vue: '@farmfe/plugin-vue', docs: '@farmfe/plugin-docs' } };
test('actual release owner delta selects native recipe without changing CI defaults', () => {
  assert.deepEqual(releaseNativeScope({ ...base, owners: '["@farmfe/plugin-vue"]' }), { core: false, create: false, plugins: ['vue'] });
  assert.deepEqual(releaseNativeScope({ ...base, owners: '[]' }), { core: false, create: false, plugins: [] });
  assert.deepEqual(releaseNativeScope({ ...base, owners: '["@farmfe/core","create-farm"]' }), { core: true, create: true, plugins: [] });
  assert.deepEqual(releaseNativeScope({ ...base, profile: 'ci' }), { core: true, create: true, plugins: ['react', 'vue'] });
  assert.throws(() => releaseNativeScope({ ...base, profile: 'ci', owners: '[]' }));
  for (const owners of ['["unknown"]', '["@farmfe/core","@farmfe/core"]', '"not an array"']) assert.throws(() => releaseNativeScope({ ...base, owners }));
  assert.deepEqual(releaseNativeScope({ ...base, owners: '["@farmfe/plugin-docs"]' }).plugins, []);
});

test('every native upload matches the selected recipe and preserves missing-file failure',()=>{
 const root=fileURLToPath(new URL('../',import.meta.url)),inventory=nativeArtifactInventory(root),names={};
 const recipe=fs.readFileSync(path.join(root,'scripts/ci-build-rust-artifacts.sh'),'utf8');
 const plugins=recipe.match(/^PLUGINS="([^"]+)"/m)[1].split(' ');
 const additional=recipe.match(/PLUGINS="\$PLUGINS ([^"]+)"/)[1].split(' ');
 for(const directory of [...plugins,...additional]) names[directory]=JSON.parse(fs.readFileSync(path.join(root,'rust-plugins',directory,'package.json'))).name;
 const supported=['win32-x64-msvc','linux-x64-gnu','linux-arm64-gnu','darwin-x64','darwin-arm64'];
 for(const file of ['rust-build-user.yaml','rust-build-release.yaml']) {
  const text=fs.readFileSync(path.join(root,'.github/workflows',file),'utf8');
  const uploads=text.split(/(?=^      - )/m).filter(block=>block.includes('uses: actions/upload-artifact@')&&/path: .*?(?:packages\/core\/binding\/|packages\/create-farm\/create-farm\.|rust-plugins\/[^/]+\/npm\/)/.test(block));
  assert.equal(uploads.length,30);
  for(const abi of [...new Set(inventory.npm.payloads.map(row=>row.platform))]) {
   const available=supported.includes(abi)?[...plugins,...additional]:plugins;
   for(const owners of [['@farmfe/core'],['create-farm'],['@farmfe/plugin-vue'],['@farmfe/plugin-docs'],[]]) {
    const selected=releaseNativeScope({profile:'release',plugins:available,names,owners:JSON.stringify(owners)});
    const expected=[...(selected.core?['@farmfe/core']:[]),...(selected.create?['create-farm']:[]),...selected.plugins.map(name=>names[name])].sort();
    const actual=[];
    for(const block of uploads) {
     const owner=block.match(/contains\(fromJSON\(inputs\.native_owners\), '([^']+)'\)/)?.[1];assert.ok(owner,'Upload is missing the native owner guard');
     assert.ok(block.includes("inputs.profile != 'release' || inputs.native_owners == ''"));assert.ok(block.includes('if-no-files-found: error'));
     const native=inventory.npm.payloads.some(row=>row.platform===abi&&row.owner===owner);
     if(additional.some(name=>names[name]===owner))assert.match(block,/\) && \(/,'Docs uploads require both ABI and owner');
     if(native&&owners.includes(owner))actual.push(owner);
    }
    assert.deepEqual(actual.sort(),expected,`${file} ${abi} ${JSON.stringify(owners)}`);
   }
  }
 }
});
