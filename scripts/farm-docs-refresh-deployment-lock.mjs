import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { lockedDeploymentPlan, websiteDependencyInputs, preflight, publicMetadata } from './farm-docs-vercel-build.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const fields = ['name', 'version', 'dependencies', 'optionalDependencies', 'peerDependencies', 'os', 'cpu', 'libc', 'exports', 'main'];
export async function createDeploymentLock(website, manifests, sourceSHA, getMetadata = publicMetadata) {
  assert.match(sourceSHA, /^[a-f0-9]{40}$/);
  const queue = Object.keys({...website.dependencies, ...website.devDependencies}).filter(name => name.startsWith('@farmfe/'));
  const closure = new Map();
  while (queue.length) {
    const name = queue.shift();
    if (closure.has(name)) continue;
    const pkg = manifests.get(name);
    assert.ok(pkg, 'Unknown source package ' + name);
    closure.set(name, pkg);
    for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies']) queue.push(...Object.keys(pkg[field] ?? {}).filter(name => name.startsWith('@farmfe/')));
  }
  const metadata = new Map();
  // Bounded registry reads, no installs, versions, publication or credentials.
  for (const [name, pkg] of closure) metadata.set(name, await getMetadata(name, pkg.version));
  await preflight({selected:closure, manifests}, name => metadata.get(name));
  const packages = [...metadata.values()].map(meta => ({...Object.fromEntries(fields.filter(field => meta[field] !== undefined).map(field => [field,meta[field]])), dist:{integrity:meta.dist.integrity,tarball:meta.dist.tarball}})).sort((a,b)=>a.name.localeCompare(b.name));
  const targets = ['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc'];
  const lock = {schemaVersion:1,registry:'https://registry.npmjs.org',sourceSHA,targets,websiteDependencies:websiteDependencyInputs(website),packages};
  for (const target of targets) lockedDeploymentPlan(website, lock, target);
  return lock;
}
async function main() {
  const source = process.argv[2];
  assert.equal(process.argv.length, 3, 'Use an exact reviewed release source SHA');
  assert.match(source ?? '', /^[a-f0-9]{40}$/);
  const git = args => execFileSync('git', args, {cwd:root,encoding:'utf8',maxBuffer:8*1024*1024});
  assert.equal(git(['rev-parse', source + '^{commit}']).trim(), source);
  const files = git(['ls-tree','-r','--name-only',source,'packages','rust-plugins']).trim().split('\n').filter(file=>/^(packages|rust-plugins)\/[^/]+\/(npm\/[^/]+\/)?package.json$/.test(file));
  const manifests = new Map(files.map(file=>{const pkg=JSON.parse(git(['show',source+':'+file]));return [pkg.name,pkg];}));
  const website = JSON.parse(fs.readFileSync(path.join(root,'website/package.json')));
  const lock = await createDeploymentLock(website,manifests,source);
  fs.writeFileSync(path.join(root,'website/farm-release-lock.json'), JSON.stringify(lock,null,2)+'\n');
  console.log(`Verified ${lock.packages.length} exact public Farm packages. Review the lock diff and run the published consumer before committing.`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) main().catch(error=>{console.error(error.message);process.exitCode=1;});
