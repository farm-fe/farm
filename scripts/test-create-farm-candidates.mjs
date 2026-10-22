import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const [abi]=process.argv.slice(2),pnpm=process.env.npm_execpath;
if(!pnpm||!fs.existsSync(pnpm)||!path.basename(pnpm).startsWith('pnpm'))throw Error('Run via pnpm run test:create-farm:candidates <ABI>');
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'farm-create-candidates-'));
const templates=['react','vue3','solid','svelte','lit','vanilla','preact','vue2'];
console.log('Create Farm candidate-package evidence: '+evidence);
if(process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT, 'evidence='+evidence+'\n');
function run(script,args){const r=spawnSync(process.execPath,[path.join(root,'scripts',script),...args],{cwd:root,stdio:'inherit',windowsHide:true});if(r.error)throw r.error;if(r.status!==0)process.exit(r.status??1);}
run('pack-create-farm-ci-candidates.mjs',[root,path.join(evidence,'packs'),pnpm,abi,...templates]);
run('test-create-farm-candidate-local.mjs',[path.join(evidence,'packs/producer.json'),path.join(root,'packages/create-farm/create-farm.js'),path.join(evidence,'templates'),pnpm,...templates]);
const result=JSON.parse(fs.readFileSync(path.join(evidence,'templates/result.json')));
if(!result.passed||result.cases.length!==templates.length)throw Error('Incomplete candidate acceptance');
console.log('All eight generated templates normally installed and built from current-job candidate tarballs. This is not registry-release acceptance.');
