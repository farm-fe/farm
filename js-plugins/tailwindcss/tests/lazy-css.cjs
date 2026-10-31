// Run against a normally installed candidate consumer; an optional built entry supports source diagnosis.
const fs = require('fs'),
  path = require('path'),
  assert = require('assert'),
  { pathToFileURL } = require('url'),
  { createRequire } = require('module');
(async () => {
  assert(
    process.argv[2] && process.argv[3],
    'Usage: node lazy-css.cjs <installed-consumer> <fresh-output-directory> [built-plugin-entry]'
  );
  const task = path.resolve(process.argv[3]),
    consumer = path.resolve(process.argv[2]),
    dir = path.join(task, 'fixture');
  assert(
    !fs.existsSync(task),
    'Use a fresh output directory to preserve prior evidence'
  );
  fs.mkdirSync(task, { recursive: true });
  assert(!fs.existsSync(dir), 'Preserve prior diagnostic');
  fs.mkdirSync(dir);
  fs.mkdirSync(path.join(dir, 'node_modules'));
  fs.mkdirSync(path.join(dir, 'node_modules/@farmfe'));
  for (const name of ['core', 'js-plugin-tailwindcss'])
    fs.symlinkSync(
      path.join(consumer, 'node_modules/@farmfe', name),
      path.join(dir, 'node_modules/@farmfe', name),
      'junction'
    );
  fs.symlinkSync(
    path.join(consumer, 'node_modules/tailwindcss'),
    path.join(dir, 'node_modules/tailwindcss'),
    'junction'
  );
  fs.writeFileSync(
    path.join(dir, 'package.json'),
    JSON.stringify({
      name: 'isolated-tailwind-lazy-regression',
      private: true,
      type: 'module'
    })
  );
  const trace = path.join(dir, 'trace.jsonl');
  fs.writeFileSync(
    path.join(dir, 'index.html'),
    '<html><head></head><body><script type="module" src="./entry.js"></script></body></html>'
  );
  fs.writeFileSync(
    path.join(dir, 'entry.js'),
    "import './style.css'; document.body.innerHTML='<button id=load>Load</button>'; document.querySelector('#load').onclick=()=>import('./lazy.js');"
  );
  fs.writeFileSync(
    path.join(dir, 'lazy.js'),
    "const el=document.createElement('div');el.id='loaded';el.className='grid md:grid-cols-2';el.innerHTML='<span>first</span><span>second</span>';document.body.append(el);"
  );
  fs.writeFileSync(path.join(dir, 'style.css'), '@import "tailwindcss";\n');
  const config = path.join(dir, 'farm.config.mjs');
  fs.writeFileSync(
    config,
    `import fs from 'node:fs';import tailwind from '${pathToFileURL(process.argv[4] ? path.resolve(process.argv[4]) : path.join(consumer, 'node_modules/@farmfe/js-plugin-tailwindcss/dist/esm/index.mjs')).href}';const trace=${JSON.stringify(trace)};const emit=x=>fs.appendFileSync(trace,JSON.stringify(x)+'\\n');const plugins=tailwind();const scan=plugins[0].transform.executor;plugins[0].transform.executor=async function(p,c){const result=await scan.call(this,p,c);const candidates=(c.readMetadataByScope('tailwindcss:candidateScope')??[]).flat();emit({hook:'scan',id:p.moduleId,grid:candidates.includes('grid'),count:candidates.length});return result};const generate=plugins[1].freezeModule.executor;plugins[1].freezeModule.executor=async function(p,c){const result=await generate.call(this,p,c);emit({hook:'generate',id:p.moduleId,grid:result?.content?.includes('.grid'),bytes:result?.content?.length});return result};plugins.push({name:'lazy-graph-trace',updateModules:{executor(p){emit({hook:'updateModules',param:p})}}});export default {root:${JSON.stringify(dir)},plugins,compilation:{input:{index:'./index.html'},lazyCompilation:true,persistentCache:false,minify:false,progress:false,sourcemap:false},server:{host:'127.0.0.1',port:4377,open:false}};`
  );
  const req = createRequire(path.join(consumer, 'package.json')),
    { chromium } = req('playwright-chromium'),
    core = await import(
      pathToFileURL(
        path.join(consumer, 'node_modules/@farmfe/core/dist/index.js')
      ).href
    );
  let server, browser;
  const report = {
    consumer,
    fixture: dir,
    success: false,
    scope:
      'Actual SDK/lazy/browser regression for new Tailwind candidates and responsive CSS; not full business acceptance'
  };
  try {
    process.chdir(dir);
    server = await core.Server.createServer({ configFile: config });
    await server.listen();
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    await page.goto('http://127.0.0.1:4377');
    await page.locator('#load').click();
    await page.locator('#loaded').waitFor();
    report.style = await page
      .locator('#loaded')
      .evaluate((el) => ({
        display: getComputedStyle(el).display,
        columns: getComputedStyle(el).gridTemplateColumns
      }));
    report.resources = Object.entries(server.getCompiler().resources())
      .filter(([name]) => name.endsWith('.css'))
      .map(([name, bytes]) => ({
        name,
        bytes: bytes.length,
        grid: Buffer.from(bytes).toString().includes('.grid')
      }));
    report.trace = fs
      .readFileSync(trace, 'utf8')
      .trim()
      .split('\n')
      .map((x) => JSON.parse(x));
    report.views = [];
    for (const width of [1440, 640]) {
      await page.setViewportSize({ width, height: 900 });
      const view = await page
        .locator('#loaded')
        .evaluate((el) => ({
          display: getComputedStyle(el).display,
          columns: getComputedStyle(el).gridTemplateColumns
        }));
      assert.strictEqual(view.display, 'grid');
      assert.strictEqual(
        view.columns.split(' ').length,
        width === 1440 ? 2 : 1
      );
      report.views.push({ width, ...view });
    }
    report.reproduced = report.style.display !== 'grid';
    assert.strictEqual(
      report.style.display,
      'grid',
      'Actual lazy browser must receive generated grid CSS'
    );
    assert(
      report.trace.some((x) => x.hook === 'generate' && x.grid),
      'CSS generator must include newly scanned grid'
    );
    report.success = true;
  } finally {
    await browser?.close();
    await server?.close();
    fs.writeFileSync(
      path.join(task, 'result.json'),
      JSON.stringify(report, null, 2)
    );
  }
  console.log(
    JSON.stringify({
      diagnosisRan: report.success,
      reproduced: report.reproduced,
      style: report.style,
      resources: report.resources
    })
  );
})().catch((e) => {
  console.error(String(e.stack || e));
  process.exitCode = 1;
});
