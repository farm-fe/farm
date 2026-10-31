import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {fileURLToPath, pathToFileURL} from 'node:url';

const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.join(repo,'e2e/fixtures/glob-compat');
const file=path.join(root,'src/files/added.js');
try {await fs.access(file);throw Error('Owned dynamic fixture already exists: inspect before running');} catch(error) {if(error.code!=='ENOENT')throw error;}
const require=createRequire(import.meta.url);
const playwright=process.env.FARM_GLOB_PLAYWRIGHT;
if(!playwright)throw Error('Set FARM_GLOB_PLAYWRIGHT to an already installed playwright-core module; no downloads');
const {chromium}=require(playwright);
const {build,Server}=await import(pathToFileURL(path.join(repo,'packages/core/dist/index.js')));
const config={root,server:{port:4367,strictPort:true,open:false},compilation:{persistentCache:false,input:{index:path.join(root,'index.html')},output:{path:path.join(root,'.output'),clean:false},resolve:{alias:{'@files':path.join(root,'src/files')}}}};
const checks=[];
let server,browser,created=false;
try {
  // Resolution normalizes configuration in place. Keep the dev input fresh.
  await build(structuredClone(config));
  await fs.access(path.join(root,'.output/index.html'));
  checks.push('production-build');
  server=await Server.createServer({...config,server:{...config.server,hmr:{host:'127.0.0.1',clientPort:4367,port:4367}},compilation:{...config.compilation,mode:'development'}});
  await server.listen();
  assert.equal(server.config.compilation.mode,'development');
  console.log('GLOB_DEV_CONFIG',JSON.stringify({mode:server.config.compilation.mode,hmr:server.config.server.hmr,runtime:server.config.compilation.runtime.plugins}));
  const watcher=server.watcher.getInternalWatcher();
  if(!watcher._readyEmitted) await new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>reject(Error('Watcher not ready')),5000);
    watcher.once('ready',()=>{clearTimeout(timer);resolve();});
  });
  watcher.on('all',(event,file)=>console.log('GLOB_WATCH',event,file));
  browser=await chromium.launch({headless:true,...(process.env.FARM_GLOB_CHROMIUM?{executablePath:process.env.FARM_GLOB_CHROMIUM}:{})});
  const page=await browser.newPage();
  page.on('websocket',socket=>socket.on('framereceived',frame=>console.log('GLOB_WS',String(frame.payload).slice(0,1500))));
  const errors=[];
  page.on('pageerror',error=>errors.push(error.message));
  page.on('response',response=>{if(response.status()>=400)console.log('GLOB_HTTP_ERROR',response.status(),response.url());});
  page.on('console',message=>console.log('GLOB_BROWSER',message.type(),message.text()));
  page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
  await page.goto('http://127.0.0.1:4367/');
  await page.waitForFunction(()=>globalThis.__globCompat);
  const state=await page.evaluate(()=>globalThis.__globCompat);
  console.log('GLOB_STATE',JSON.stringify(state));
  assert.equal(state.eager['./files/a.js'],'a');
  assert.equal(state.eager['./files/c.ts'],'typescript');
  assert.deepEqual(state.lazyKeys,['./files/a.js']);
  assert.deepEqual(state.lazyValues,['a']);
  assert.deepEqual(state.query,{'./files/a.js':'alpha'});
  assert.equal(state.based['./a.js'],'a');
  assert.equal(state.rooted['/src/files/a.js'],'a');
  assert.equal(state.alias['/src/files/a.js'],'a');
  assert.equal(state.exhaustive['./files/.hidden.js'],'hidden');
  checks.push('browser-adapter-matrix');
  await fs.writeFile(file,"export default 'added';\n",{flag:'wx'});created=true;
  await page.waitForFunction(()=>globalThis.__globCompat?.lazyKeys.includes('./files/added.js'),undefined,{timeout:15000});
  checks.push('hmr-add');
  await fs.unlink(file);created=false;
  await page.waitForFunction(()=>!globalThis.__globCompat?.lazyKeys.includes('./files/added.js'),undefined,{timeout:15000});
  checks.push('hmr-delete');
  assert.deepEqual(errors,[]);
  checks.push('no-console-or-runtime-errors');
  await browser.close();browser=undefined;
  await server.close();server=undefined;
  const warmConfig=()=>({...structuredClone(config),compilation:{...structuredClone(config.compilation),mode:'production',persistentCache:{cacheDir:path.join(root,'.cache'),namespace:'glob-compat-v1'},output:{path:path.join(root,'.warm-output'),clean:false}}});
  const emittedScripts=async()=>{
    const html=await fs.readFile(path.join(root,'.warm-output/index.html'),'utf8');
    const scripts=[...html.matchAll(/<script[^>]+src=["']?([^\s"'>]+)/g)].map(match=>path.basename(match[1]));
    return (await Promise.all(scripts.map(script=>fs.readFile(path.join(root,'.warm-output',script),'utf8')))).join('\n');
  };
  await build(warmConfig());
  assert(!(await emittedScripts()).includes('./files/added.js'));
  await fs.writeFile(file,"export default 'added';\n",{flag:'wx'});created=true;
  await build(warmConfig());
  assert((await emittedScripts()).includes('./files/added.js'));
  await fs.unlink(file);created=false;
  await build(warmConfig());
  assert(!(await emittedScripts()).includes('./files/added.js'));
  checks.push('warm-cache-add-delete-membership');
  console.log(JSON.stringify({status:'passed',checks,platform:process.platform}));
} finally {
  if(created) {
    const current=await fs.readFile(file,'utf8');
    if(current!=="export default 'added';\n")throw Error('Dynamic fixture changed unexpectedly; preserving it');
    await fs.unlink(file);
  }
  await browser?.close();
  await server?.close();
}
