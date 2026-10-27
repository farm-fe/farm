// cspell:words statfs bavail
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export function nativeArtifactRoute(root, pool, sourceSHA, pkg, plan) {
  assert.match(sourceSHA, /^[a-f0-9]{40}$/);
  const owner = plan.npm.primary.find(row => row.name === pkg.owner);
  assert.ok(owner, 'Missing native owner');
  const ownerDirectory = path.dirname(owner.manifestPath).replaceAll('\\', '/');
  const suffix = pkg.owner === '@farmfe/core' ? '' : pkg.owner === 'create-farm' ? '-create-farm' : `-plugin-${ownerDirectory.split('/')[1]}`;
  const manifest = path.resolve(root, pkg.manifestPath);
  assert.ok(manifest.startsWith(path.resolve(root) + path.sep), 'Unsafe native manifest');
  const main = JSON.parse(fs.readFileSync(manifest, 'utf8')).main;
  assert.ok(typeof main === 'string' && path.basename(main) === main && !main.includes('\\'), 'Unsafe native main');
  assert.match(pkg.platform, /^(linux|darwin|win32)-[a-z0-9-]+$/);
  const artifact = `${sourceSHA}-${pkg.platform}${suffix}`;
  return { name: pkg.name, version: pkg.version, platform: pkg.platform, owner: pkg.owner,
    artifact, source: path.join(pool, artifact, main), destination: path.join(path.dirname(manifest), main) };
}
function hashFile(file) {
  const fd = fs.openSync(file, 'r');
  const buffer = Buffer.alloc(65536), hash = crypto.createHash('sha256');
  try { for (;;) { const count = fs.readSync(fd, buffer, 0, buffer.length, null); if (!count) break; hash.update(buffer.subarray(0, count)); } }
  finally { fs.closeSync(fd); }
  return hash.digest('hex');
}
export function assembleNativeArtifacts(root, pool, sourceSHA, plan) {
  const routes = plan.npm.payloads.map(pkg => nativeArtifactRoute(root, pool, sourceSHA, pkg, plan));
  const realPool = fs.realpathSync(pool), realRoot = fs.realpathSync(root);
  let bytes = 0;
  // Validate every required artifact before changing the candidate source tree.
  for (const route of routes) {
    const stat = fs.lstatSync(route.source);
    assert.ok(stat.isFile() && !stat.isSymbolicLink(), `Missing regular native artifact: ${route.artifact}`);
    assert.ok(fs.realpathSync(route.source).startsWith(realPool + path.sep), 'Artifact symlink escapes its pool');
    assert.ok(fs.realpathSync(path.dirname(route.destination)).startsWith(realRoot + path.sep), 'Native destination escapes candidate source');
    route.sha256 = hashFile(route.source); route.bytes = stat.size; bytes += stat.size;
    if (fs.existsSync(route.destination)) assert.equal(hashFile(route.destination), route.sha256, `Refusing to overwrite different existing native bytes: ${route.name}`);
  }
  const disk = fs.statfsSync(root);
  assert.ok(disk.bavail * disk.bsize - bytes >= 8 * 1024 ** 3, 'Native assembly disk guard');
  for (const route of routes) {
    fs.copyFileSync(route.source, route.destination);
    assert.equal(hashFile(route.destination), route.sha256, 'Native bytes changed while assembling');
  }
  return routes.map(({ source, destination, ...record }) => ({ ...record, file: path.relative(root, destination).replaceAll('\\', '/') }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [pool, output, explicitRoot, explicitSHA] = process.argv.slice(2);
  const root = path.resolve(explicitRoot ?? fileURLToPath(new URL('../', import.meta.url)));
  const sourceSHA = explicitSHA ?? process.env.GITHUB_SHA;
  if (!pool || !output) throw new Error('Expected artifact directory and assembly receipt');
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const records = assembleNativeArtifacts(root, path.resolve(pool), sourceSHA, plan);
  fs.writeFileSync(output, JSON.stringify({ schemaVersion: 1, sourceCommit: sourceSHA, records, count: records.length, published: false }, null, 2));
  console.log(JSON.stringify({ assembled: records.length, sourceCommit: sourceSHA, published: false }));
}
