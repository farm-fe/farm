import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const sha=value=>createHash('sha256').update(value).digest('hex');
export function validateSvg(svg){
 assert.match(svg,/<svg\b/,'SVG root is required');
 assert(!/<\s*(?:script|foreignObject)\b|\son[a-z]+\s*=|<!DOCTYPE|<!ENTITY/i.test(svg),'Unsafe SVG structure');
 assert(![...svg.matchAll(/(?:href|xlink:href)="([^"]*)"/g)].some(m=>!m[1].startsWith('#')),'External SVG reference');
 assert(![...svg.matchAll(/url\(\s*["']?([^"')\s]+)/gi)].some(m=>!m[1].startsWith('#')),'External SVG stylesheet resource');
}
export async function checkDiagrams(folder=path.join(root,'website/static/diagrams')){
 const manifest=JSON.parse(await fs.readFile(path.join(folder,'manifest.json'),'utf8'));
 assert.equal(manifest.schemaVersion,1);assert.equal(manifest.authoringPackage,'@mermaid-js/mermaid-cli@12.0.0');assert.equal(manifest.renderer,'mermaid@12.2.0');
 const seen=new Set();let outputs=0;
 for(const item of manifest.diagrams){
  assert.match(item.source,/^[a-z0-9-]+\.mmd$/,'Unsafe source filename');assert(!seen.has(item.source),'Duplicate source');seen.add(item.source);
  const source=await fs.readFile(path.join(folder,item.source));assert.equal(sha(source),item.sourceSha256,`Stale Mermaid source: ${item.source}`);
  for(const mode of ['light','dark']){const output=item.outputs[mode];assert.equal(output.file,item.source.replace('.mmd',`-${mode}.svg`));const svg=await fs.readFile(path.join(folder,output.file),'utf8');assert.equal(sha(svg),output.sha256,`Stale SVG: ${output.file}`);validateSvg(svg);outputs++;}
 }
 assert.deepEqual([...seen].sort(),(await fs.readdir(folder)).filter(n=>n.endsWith('.mmd')).sort(),'All Mermaid sources need manifest entries');
 return{sources:seen.size,outputs};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))console.log(JSON.stringify(await checkDiagrams()));
