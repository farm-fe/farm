import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {test} from 'node:test';
const root=new URL('../',import.meta.url);
const json=async p=>JSON.parse(await fs.readFile(new URL(p,root),'utf8'));
const docs={en:'website/docs',zh:'website/i18n/zh/docusaurus-plugin-content-docs/current'};
test('ecosystem navigation has bilingual current pages without replacing Farm Docs routes',async()=>{
 const site=await json('website/farm-docs.site.json');
 assert.equal(site.theme.navbar.find(n=>n.label==='Ecosystem').to,'/docs/ecosystem/overview');
 const presentation=await json('website/farm-docs.presentation.json');
 assert.deepEqual(presentation.navigation.sections.find(n=>n.id==='Ecosystem').prefixes,['ecosystem','farm-docs']);
 for(const [language,folder]of Object.entries(docs)){
  const sidebar=await json(`website/sidebars/${language}-current.json`);const ids=[];
  function collect(items){for(const item of items){if(item.type==='doc')ids.push(item.id);else if(item.items)collect(item.items);}}
  collect(sidebar.items);assert.equal(ids.length,new Set(ids).size);
  for(const id of ['ecosystem/overview','ecosystem/rust-crates','farm-docs/overview']){assert(ids.includes(id));await fs.access(new URL(`${folder}/${id}.md`,root));}
 }
});
test('both catalogues list actual ecosystem crate names and manifest versions',async()=>{
 const folders=await fs.readdir(new URL('rust-ecosystem/',root),{withFileTypes:true});let count=0;
 for(const folder of folders.filter(n=>n.isDirectory())){
  let source;try{source=await fs.readFile(new URL(`rust-ecosystem/${folder.name}/Cargo.toml`,root),'utf8');}catch{continue;}
  const name=source.match(/^name\s*=\s*"([^"]+)"/m)?.[1];const version=source.match(/^version\s*=\s*"([^"]+)"/m)?.[1];assert(name&&version);count++;
  for(const base of Object.values(docs)){
   const content=await fs.readFile(new URL(`${base}/ecosystem/rust-crates.md`,root),'utf8');
   assert(content.includes(`### ${name}`));assert(content.includes('`'+version+'`'));assert(content.includes(`/rust-ecosystem/${folder.name}`));
   assert(content.includes('2026-10-24'));assert(content.includes('crates.io/farmfe')===false,'Registry URLs must use actual /crates/ path');
  }
 }
 assert.equal(count,10);
});
