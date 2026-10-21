import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync, execFileSync} from 'node:child_process';

// A candidate-package acceptance test, deliberately distinct from registry acceptance.
const [producerArg, generatorArg, destinationArg, pnpmArg, ...templates] = process.argv.slice(2);
const [producerFile, generator, destination, pnpm] = [producerArg,generatorArg,destinationArg,pnpmArg].map(value=>value && path.resolve(value));
if (!producerFile || !generator || !destination || !pnpm || !templates.length) {
  throw Error('Usage: producer.json generator.js fresh-output pnpm.cjs template...');
}
const producer = JSON.parse(fs.readFileSync(producerFile));
if(producer.state !== 'packed' || !/^[a-f0-9]{40}$/.test(producer.head)) throw Error('Completed source-bound candidate pack inventory required');
function satisfiesTemplate(version, selector) {
  if(selector !== '^2.0.0') throw Error('Review new template candidate constraint: '+selector);
  return /^2\.\d+\.\d+$/.test(version);
}
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
if (fs.existsSync(destination)) throw Error('Output already exists; preserve prior evidence');
fs.mkdirSync(destination, {recursive: true});
const rows = [...producer.roots, ...producer.platforms];
const packages = new Map();
for (const row of rows) {
  const archive = path.resolve(row.archive);
  if (hash(fs.readFileSync(archive)) !== row.sha256) throw Error('Archive changed: ' + row.name);
  const manifest = JSON.parse(execFileSync('tar', ['-xOf', archive, 'package/package.json'], {encoding: 'utf8'}));
  if (manifest.name !== row.name || manifest.version !== row.version || packages.has(row.name)) throw Error('Archive identity mismatch');
  packages.set(row.name, {...row, archive, manifest});
}
const state = {sourceHead: producer.head, generator: path.resolve(generator), generatorSha256: hash(fs.readFileSync(generator)),
  acceptance: 'local candidate tarballs; not published-registry acceptance', packages: rows.map(({name,version,sha256})=>({name,version,sha256})),
  started: new Date().toISOString(), cases: [], published: false};
const save = () => fs.writeFileSync(path.join(destination, 'result.json'), JSON.stringify(state, null, 2));
function run(args, cwd, log) {
  const fd = fs.openSync(log, 'wx');
  let result;
  try {result = spawnSync(process.execPath, args, {cwd, env: {...process.env, PATH: path.dirname(process.execPath)+path.delimiter+process.env.PATH}, windowsHide: true, stdio: ['ignore', fd, fd], timeout: 600000});}
  finally {fs.closeSync(fd);}
  if (result.error) throw result.error;
  if (result.status !== 0) throw Error(`Exit ${result.status}: ${log}`);
}
try {
  for (const template of templates) {
    if (!/^[a-z0-9-]+$/.test(template)) throw Error('Invalid template');
    const row = {template, started: new Date().toISOString(), passed: false}; state.cases.push(row); save();
    const app = path.join(destination, 'app-' + template);
    run([generator, path.basename(app), '--template', template], destination, path.join(destination, template+'-generate.log'));
    const manifestPath = path.join(app, 'package.json');
    const bytes = fs.readFileSync(manifestPath), manifest = JSON.parse(bytes);
    fs.writeFileSync(path.join(destination, template+'-generated-package.json'), bytes);
    row.originalManifestSha256 = hash(bytes);
    if (manifest.pnpm || manifest.overrides || manifest.resolutions) throw Error('Review existing template resolver configuration');
    for (const section of ['dependencies','devDependencies','optionalDependencies','peerDependencies']) {
      for (const [name, spec] of Object.entries(manifest[section] ?? {})) {
        if (!name.startsWith('@farmfe/')) continue;
        const candidate = packages.get(name);
        if (!candidate || !satisfiesTemplate(candidate.version, spec)) throw Error(`Candidate does not satisfy unchanged template: ${name}@${spec}`);
      }
    }
    // Test-only resolver substitution; all original dependency constraints remain intact.
    manifest.pnpm = {overrides: Object.fromEntries([...packages].map(([name, p]) => [name, 'file:'+p.archive.replaceAll('\\','/')]))};
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2)+'\n');
    run([pnpm, 'install', '--config.engine-strict=true'], app, path.join(destination, template+'-install.log'));
    const installedManifest=JSON.parse(fs.readFileSync(manifestPath));
    for(const section of ['dependencies','devDependencies','optionalDependencies','peerDependencies']) {
      if(JSON.stringify(installedManifest[section])!==JSON.stringify(manifest[section])) throw Error('Installation changed template dependency constraints');
    }
    for (const section of ['dependencies','devDependencies']) for (const name of Object.keys(manifest[section] ?? {})) {
      if (!name.startsWith('@farmfe/')) continue;
      const installed = JSON.parse(fs.readFileSync(path.join(app, 'node_modules', ...name.split('/'), 'package.json')));
      if (installed.version !== packages.get(name).version) throw Error('Installed candidate mismatch: '+name);
    }
    run([pnpm, 'build'], app, path.join(destination, template+'-build.log'));
    if (!fs.existsSync(path.join(app, 'dist', 'index.html'))) throw Error('No real built page');
    row.lockSha256 = hash(fs.readFileSync(path.join(app, 'pnpm-lock.yaml')));
    row.passed = true; row.ended = new Date().toISOString(); save();
  }
  state.passed = true;
} catch(error) {state.passed = false; state.error = String(error.stack ?? error); process.exitCode = 1;}
state.ended = new Date().toISOString(); save();
