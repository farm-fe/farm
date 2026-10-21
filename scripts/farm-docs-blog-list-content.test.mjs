import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {test} from 'node:test';

test('Rust full blog listings retain original article identities without copying their source',async()=>{
 const parent=path.resolve('examples/docs-minimal');
 const root=await fs.mkdtemp(path.join(parent,'blog-list-content-'));
 try{
  await fs.mkdir(path.join(root,'blog'));
  await fs.writeFile(path.join(root,'blog/index.md'),'# Blob\n\n## 2024\n');
  const {default:docs}=await import('../rust-plugins/docs/scripts/func.js');
  const {resolveConfig,createCompiler}=await import('../packages/core/dist/index.js');
  const site={title:'Full blog',url:'https://example.com',blog:{collections:[{id:'news',dir:'blog',routeBase:'/blog/',blogListContent:'full'}]}};
  const compiler=createCompiler(await resolveConfig({root,configFile:false,compilation:{input:{site:'docs:site'},output:{path:'output',targetEnv:'node',format:'cjs'},persistentCache:false,minify:false,sourcemap:false,lazyCompilation:false},plugins:[docs({site,prepare:true})]},'build','production'));
  await compiler.compile();
  const plan=JSON.parse(compiler.resources()['docs-plan.json'].toString());
  const article=plan.routes.find(route=>route.page.id==='index').page;
  const listing=plan.routes.find(route=>route.page.kind==='blog-index').page;
  assert.equal(listing.blogView.fullContent,true);
  assert.deepEqual(listing.blogView.posts,[article.key]);
  assert.equal(plan.routes.find(route=>route.page.kind==='blog-archive').page.blogView.fullContent,undefined,'archive remains a link index');
  assert.equal(plan.modules.filter(module=>path.resolve(module.sourcePath)===path.join(root,'blog/index.md')).length,1,'original author module is not copied into synthetic Markdown');
 }finally{
  assert.equal(path.dirname(root),parent);assert.ok(path.basename(root).startsWith('blog-list-content-'));
  await fs.rm(root,{recursive:true,force:true});
 }
});

test('full blog listing renders original MDX in static HTML and lazy client navigation',{timeout:120000},async()=>{
 const parent=path.resolve('examples/docs-minimal');
 const root=await fs.mkdtemp(path.join(parent,'blog-list-content-'));
 let preview,browser;
 try{
  for(const dir of ['blog','docs','src'])await fs.mkdir(path.join(root,dir));
  await fs.writeFile(path.join(root,'src/post.css'),'.private-blog-identity { color: rgb(12, 34, 56); }\n');
  await fs.writeFile(path.join(root,'src/shared.css'),'.shared-listing { font-weight: 700; }\n');
  await fs.writeFile(path.join(root,'src/second.css'),'.second-private { background-color: rgb(65, 43, 21); }\n');
  await fs.writeFile(path.join(root,'src/Identity.tsx'),"import './post.css'; import './shared.css'; import {usePage} from '@farmfe/docs/client'; export default function Identity(){return <p className='private-blog-identity shared-listing' data-testid='post-identity'>{usePage().routePath}</p>}\n");
  await fs.writeFile(path.join(root,'src/Second.tsx'),"import './shared.css'; import './second.css'; import {usePage} from '@farmfe/docs/client'; export default function Second(){return <p className='second-private shared-listing' data-testid='second-identity'>{usePage().routePath}</p>}\n");
  await fs.writeFile(path.join(root,'docs/about.md'),'# About\n\n[Articles](/guide/blog/)\n');
  await fs.writeFile(path.join(root,'blog/asset.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="17" height="17"><rect width="17" height="17"/></svg>');
  await fs.writeFile(path.join(root,'blog/release.mdx'),"---\ntitle: Release\ndate: 2025-01-02\n---\nimport Identity from '../src/Identity.tsx';\n\n# Release\n\n## 2024\n\nOriginal listing body.\n\n<Identity />\n\n[About](/guide/docs/about/#about)\n\n![Original asset](./asset.svg)\n");
  await fs.writeFile(path.join(root,'blog/second.mdx'),"---\ntitle: Second\ndate: 2024-01-02\n---\nimport Second from '../src/Second.tsx';\n\n# Second\n\nSecond original body.\n\n<Second />\n");
  const site={title:'Full blog',url:'https://example.com',basePath:'/guide/',docs:{collections:[{id:'docs',dir:'docs',routeBase:'/docs/'}]},blog:{collections:[{id:'news',dir:'blog',routeBase:'/blog/',blogListContent:'full'}]}};
  const {run}=await import('../packages/docs/dist/workflow.js');
  await run({command:'build',config:{root},options:{site,farm:{compilation:{output:{path:'build'},persistentCache:false,sourcemap:false}}}});
  assert.match(await fs.readFile(path.join(root,'build/blog/index.html'),'utf8'),/Original listing body/,'listing physical HTML contains original body');
  const listingHtml=await fs.readFile(path.join(root,'build/blog/index.html'),'utf8');
  assert.match(listingHtml,/<time dateTime="2025-01-02">2025-01-02<\/time>|<time datetime="2025-01-02">2025-01-02<\/time>/,'full listing retains original publication date in static HTML');
  const manifest=JSON.parse(await fs.readFile(path.join(root,'build/docs-manifest.json'),'utf8'));
  const report=JSON.parse(await fs.readFile(path.join(root,'build/docs-build-report.json'),'utf8'));
  const resources=route=>manifest.pageResources[report.pages.find(page=>page.routePath===route).key];
  const listingStyles=resources('/guide/blog/').styles;
  assert.equal(new Set(listingStyles).size,listingStyles.length,'shared listing styles are deduplicated');
  for(const route of ['/guide/blog/release/','/guide/blog/second/'])for(const style of resources(route).styles)assert.ok(listingStyles.includes(style),'full listing includes actual article style: '+style);
  const {startDocsPreview}=await import('../packages/docs/dist/preview.js');
  preview=await startDocsPreview({root,compilation:{output:{path:'build'}},server:{preview:{host:'127.0.0.1',port:0}}});
  const {chromium}=await import('playwright-chromium');browser=await chromium.launch({headless:true});
  const errors=[];
  for(const javaScriptEnabled of [false,true]){
   const context=await browser.newContext({javaScriptEnabled});const page=await context.newPage();page.setDefaultTimeout(10000);
   page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
   await page.goto(new URL('/guide/docs/about/',preview.url).href);
   let identity;
   if(javaScriptEnabled){await page.waitForFunction(()=>typeof history.state?.__farmDocsOwner==='string');identity=await page.evaluate(()=>window.__blogListIdentity=crypto.randomUUID());}
   await page.getByRole('link',{name:'Articles',exact:true}).click();
   await page.getByRole('heading',{name:'2024',exact:true}).waitFor();
   assert.equal(await page.getByText('Original listing body.',{exact:true}).count(),1);
   assert.equal(await page.locator('article time[datetime="2025-01-02"]').textContent(),'2025-01-02','publication date survives no-JS and lazy navigation');
   assert.equal(await page.getByTestId('post-identity').textContent(),'/guide/blog/release/');
   assert.equal(await page.getByTestId('post-identity').evaluate(element=>getComputedStyle(element).color),'rgb(12, 34, 56)','private article CSS is active on full listing, including no-JS');
   assert.equal(await page.getByTestId('post-identity').evaluate(element=>getComputedStyle(element).fontWeight),'700');
   assert.equal(await page.getByTestId('second-identity').textContent(),'/guide/blog/second/');
   assert.equal(await page.getByTestId('second-identity').evaluate(element=>getComputedStyle(element).backgroundColor),'rgb(65, 43, 21)');
   assert.equal(await page.getByTestId('second-identity').evaluate(element=>getComputedStyle(element).fontWeight),'700');
   const image=page.getByRole('img',{name:'Original asset',exact:true});
   assert.ok(await image.evaluate(async element=>{await element.decode();return element.naturalWidth===17;}));
   const about=page.locator('article').getByRole('link',{name:'About',exact:true});
   assert.equal(await about.getAttribute('href'),'/guide/docs/about/#about');
   if(javaScriptEnabled)assert.equal(await page.evaluate(()=>window.__blogListIdentity),identity);
   await about.click();await page.getByRole('heading',{name:'About',exact:true}).waitFor();
   await page.getByRole('link',{name:'Articles',exact:true}).click();await page.getByTestId('post-identity').waitFor();
   assert.equal(await page.getByTestId('post-identity').evaluate(element=>getComputedStyle(element).color),'rgb(12, 34, 56)','private CSS survives leaving and returning to the listing');
   assert.equal(await page.getByTestId('second-identity').evaluate(element=>getComputedStyle(element).backgroundColor),'rgb(65, 43, 21)');
   await context.close();
  }
  assert.deepEqual(errors,[]);
 }finally{
  await browser?.close();await preview?.close();
  assert.equal(path.dirname(root),parent);assert.ok(path.basename(root).startsWith('blog-list-content-'));
  await fs.rm(root,{recursive:true,force:true});
 }
});

test('full listing development refreshes article dependencies, discovers posts and repairs invalid MDX',{timeout:120000},async()=>{
 const parent=path.resolve('examples/docs-minimal'),root=await fs.mkdtemp(path.join(parent,'blog-list-content-'));
 let handle,browser;
 try{
  await fs.mkdir(path.join(root,'blog'));await fs.mkdir(path.join(root,'src'));
  const source=path.join(root,'blog/release.mdx'),component=path.join(root,'src/Probe.tsx'),added=path.join(root,'blog/second.md');
  const body=(heading,id='old')=>`---\nid: ${id}\ntitle: Release\nslug: release\n---\nimport Probe from '../src/Probe.tsx';\n\n# Release\n\n${heading}\n\n<Probe />\n`;
  const dependency=text=>`import {usePage} from '@farmfe/docs/client'; export default function Probe(){return <><p data-testid='listing-probe'>${text}</p><p data-testid='listing-id'>{usePage().id}</p></>}\n`;
  await fs.writeFile(component,dependency('Original dependency'));
  await fs.writeFile(source,body('Original listing paragraph.'));
  const {startDocsDevelopment}=await import('../packages/docs/dist/index.js');
  handle=await startDocsDevelopment({title:'Full blog dev',url:'https://example.com',blog:{collections:[{id:'news',dir:'blog',routeBase:'/blog/',blogListContent:'full'}]}},{root,server:{host:'127.0.0.1',port:0,hmr:{port:0},open:false},compilation:{persistentCache:false,sourcemap:false}});
  const {chromium}=await import('playwright-chromium');browser=await chromium.launch({headless:true});
  const page=await browser.newPage();page.setDefaultTimeout(15000);const frames=[];
  page.on('websocket',socket=>socket.on('framereceived',frame=>{const payload=String(frame.payload);if(payload.includes('"type":"error"'))frames.push(payload);}));
  await page.goto(new URL('/blog/',handle.url).href);
  await page.getByText('Original listing paragraph.',{exact:true}).waitFor();await page.getByTestId('listing-probe').waitFor();
  const identity=await page.evaluate(()=>window.__fullListingDevIdentity=crypto.randomUUID());
  await fs.writeFile(component,dependency('Updated dependency'));
  await page.getByTestId('listing-probe').getByText('Updated dependency',{exact:true}).waitFor();
  await fs.writeFile(source,body('Revised listing paragraph.'));await page.getByText('Revised listing paragraph.',{exact:true}).waitFor();
  await fs.writeFile(added,'---\ndate: 2025-02-03\n---\n# Second post\n\nAdded listing paragraph.\n',{flag:'wx'});
  await page.getByRole('heading',{name:'Second post',exact:true}).waitFor();
  await fs.unlink(added);await page.getByRole('heading',{name:'Second post',exact:true}).waitFor({state:'detached'});
  await fs.writeFile(source,'import {\n');
  for(let index=0;index<60&&!frames.length;index++)await new Promise(resolve=>setTimeout(resolve,250));
  assert.ok(frames.length,'native MDX error is reported while full listing retains the last good document');
  assert.ok(await page.getByText('Revised listing paragraph.',{exact:true}).isVisible());
  await fs.writeFile(source,body('Repaired listing paragraph.'));await page.getByText('Repaired listing paragraph.',{exact:true}).waitFor();
  await page.getByTestId('listing-id').getByText('old',{exact:true}).waitFor();
  await fs.writeFile(source,body('Repaired listing paragraph.','new'));
  await page.getByTestId('listing-id').getByText('new',{exact:true}).waitFor();
  assert.equal(await page.getByTestId('listing-probe').count(),1);assert.equal(await page.evaluate(()=>window.__fullListingDevIdentity),identity);
 }finally{
  await browser?.close();await handle?.close();
  assert.equal(path.dirname(root),parent);assert.ok(path.basename(root).startsWith('blog-list-content-'));
  await fs.rm(root,{recursive:true,force:true});
 }
});
