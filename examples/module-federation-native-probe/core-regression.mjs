import {build} from '@farmfe/core';
import {chromium} from 'playwright-chromium';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import http from 'node:http';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const base=path.dirname(fileURLToPath(import.meta.url));
const root=path.join(base,'core-cases'),dist=path.join(base,'dist/core-cases');
const config={root,configFile:false,mode:'production',compilation:{input:{index:'./index.html'},output:{path:dist,targetEnv:'browser-esnext'},persistentCache:false,lazyCompilation:false,sourcemap:false,presetEnv:false}};
await build(config);
const server=http.createServer(async(req,res)=>{try{const file=path.join(dist,new URL(req.url,'http://localhost').pathname==='/'?'index.html':req.url);const body=await fs.readFile(file);res.setHeader('Content-Type',file.endsWith('.html')?'text/html':'text/javascript');res.end(body);}catch{res.statusCode=404;res.end();}});
await new Promise(resolve=>server.listen(4190,'127.0.0.1',resolve));
let browser;const evidence={};
try {
 browser=await chromium.launch({headless:true});const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://127.0.0.1:4190');await page.waitForFunction(()=>window.result,{}, {timeout:10000});
 evidence.result=await page.evaluate(()=>window.result);assert.deepEqual(evidence.result,{sync:7,cycle:{name:'a',peer:'b'},esm:42,consumer:42,caught:'expected async failure',iife:42,order:['before','after']});assert.deepEqual(errors,[]);
 const invalidConfig={...config,compilation:{...config.compilation,input:{invalid:'./invalid.cjs'},output:{...config.compilation.output,path:path.join(base,'dist/core-invalid')}}};
 const invalid = spawnSync(process.execPath,['--input-type=module','-e',`import {build} from '@farmfe/core'; await build(${JSON.stringify(invalidConfig)});`],{cwd:base,encoding:'utf8'});
 assert.notEqual(invalid.status,0);const diagnostic=invalid.stdout+invalid.stderr;
 assert.match(diagnostic,/Cannot synchronously require an asynchronous module/);evidence.diagnostic=diagnostic;
 invalidConfig.compilation.input={cycle:'./async-cycle-a.cjs'};
 const cyclic=spawnSync(process.execPath,['--input-type=module','-e',`import {build} from '@farmfe/core'; await build(${JSON.stringify(invalidConfig)});`],{cwd:base,encoding:'utf8',timeout:15000});
 assert.notEqual(cyclic.status,0);assert.match(cyclic.stdout+cyclic.stderr,/static dependency cycle/);evidence.asyncCycleDiagnostic=cyclic.stdout+cyclic.stderr;evidence.passed=true;
} catch(e) {evidence.passed=false;evidence.error=e.stack;process.exitCode=1;}
finally {await browser?.close();await new Promise(resolve=>server.close(resolve));await fs.writeFile(path.join(base,'evidence-core-regression.json'),JSON.stringify(evidence,null,2));console.log(JSON.stringify(evidence,null,2));}
