import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import crypto from 'node:crypto';
import { archiveFiles, sha256 } from './release-archive.mjs';
import { publicationNpm } from './v2-release-inventory.mjs';

const integrity = bytes => `sha512-${crypto.createHash('sha512').update(bytes).digest('base64')}`;
const packageIdentity = row => `${row.name}@${row.version}`;
// Exact target spelling follows the Core NAPI loader and declared payloads.
export function consumerHostAbi({ platform = process.platform, arch = process.arch, report = () => process.report.getReport() } = {}) {
  if (platform === 'win32' && ['x64', 'arm64', 'ia32'].includes(arch)) return `win32-${arch}-msvc`;
  if (platform === 'darwin' && ['x64', 'arm64'].includes(arch)) return `darwin-${arch}`;
  if (platform === 'linux' && ['x64', 'arm64'].includes(arch)) {
    const header = report()?.header;
    assert.ok(header && typeof header === 'object', 'Linux libc report is unavailable');
    const glibc = header.glibcVersionRuntime;
    assert.ok(glibc === undefined || (typeof glibc === 'string' && glibc.length > 0), 'Invalid Linux libc identity');
    return `linux-${arch}-${glibc ? 'gnu' : 'musl'}`;
  }
  throw new Error(`Unsupported consumer platform: ${platform}/${arch}`);
}
export async function candidateNpmEvidence(plan, receipt, directory) {
  assert.equal(receipt.planSha256, sha256(Buffer.from(JSON.stringify(plan))), 'Candidate plan identity mismatch');
  const selected = publicationNpm(plan), expected = [...selected.primary, ...selected.payloads];
  assert.deepEqual((receipt.records ?? []).map(packageIdentity).sort(), expected.map(packageIdentity).sort(), 'Incomplete sealed npm family/delta');
  const evidence = new Map();
  for (const row of receipt.records) {
    assert.equal(path.basename(row.file), row.file, 'Unsafe candidate archive path');
    const file = path.join(directory, row.file), stat = await fs.lstat(file);
    assert.ok(stat.isFile() && !stat.isSymbolicLink(), 'Candidate archive must be a regular file');
    const bytes = await fs.readFile(file); assert.equal(sha256(bytes), row.sha256, `Candidate bytes differ: ${row.name}`);
    const manifest = JSON.parse(archiveFiles(bytes).get('package/package.json'));
    assert.equal(manifest.name, row.name); assert.equal(manifest.version, row.version);
    evidence.set(row.name, { ...row, integrity: integrity(bytes), manifest, archive: file, kind: 'sealed' });
  }
  for (const row of plan.publication?.reuseNpm ?? []) {
    assert.match(row.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/);
    assert.ok(!evidence.has(row.name), 'Sealed/reuse identity collision');
    evidence.set(row.name, { ...row, kind: 'registry-reuse' });
  }
  for (const row of [...plan.npm.primary, ...plan.npm.payloads]) {
    const found = evidence.get(row.name);
    assert.ok(found && found.version === row.version, `Missing exact family install evidence: ${row.name}`);
  }
  return evidence;
}
export function candidateInstallArgs(npm, registry) {
  return [npm, 'install', '--ignore-scripts', '--no-audit', '--no-fund', '--replace-registry-host=never', `--registry=${registry}`];
}
export function verifyInstalledNpmLock(plan, manifest, lock, evidence, { registry }) {
  const registryURL = new URL(registry);
  assert.ok(lock.lockfileVersion >= 2 && lock.packages, 'Actual npm lock required');
  for (const [key, row] of Object.entries(lock.packages)) {
    assert.ok(!row.link, `Workspace/local link in isolated install: ${key}`);
    if (row.resolved) {
      const url = new URL(row.resolved);
      assert.ok(url.origin === registryURL.origin || url.origin === 'https://registry.npmjs.org', 'Unexpected package origin');
    }
  }
  for (const [name, version] of Object.entries(manifest.dependencies)) {
    const row = lock.packages[`node_modules/${name}`], expected = evidence.get(name);
    assert.ok(expected && expected.version === version, `Missing precise consumer evidence: ${name}`);
    assert.equal(row?.version, version, `Installed primary version differs: ${name}`);
    assert.equal(row.integrity, expected.integrity, `Installed immutable bytes differ: ${name}`);
  }
  for (const pkg of plan.npm.payloads) {
    const row = lock.packages[`node_modules/${pkg.name}`];
    if (row) {
      assert.equal(row.version, pkg.version);
      assert.equal(row.integrity, evidence.get(pkg.name).integrity, `Installed native bytes differ: ${pkg.name}`);
    }
  }
}
export async function readRegistryBody(response, limit = 32 * 1024 * 1024) {
  const declared = response.headers.get('content-length');
  if (declared !== null) assert.ok(Number.isSafeInteger(Number(declared)) && Number(declared) >= 0 && Number(declared) <= limit, 'Registry metadata too large');
  const chunks = []; let size = 0;
  assert.ok(response.body, 'Registry metadata body missing');
  for await (const chunk of response.body) {
    size += chunk.byteLength; assert.ok(size <= limit, 'Registry metadata too large');
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks, size);
}
export async function serveCandidateRegistry(evidence, { fetcher = fetch } = {}) {
  const server = http.createServer(async (request, response) => {
    try {
      assert.equal(request.method, 'GET', 'Read-only candidate registry');
      const url = new URL(request.url, 'http://localhost');
      assert.equal(url.search, '', 'Unexpected registry query');
      if (url.pathname.startsWith('/archives/')) {
        const name = decodeURIComponent(url.pathname.slice('/archives/'.length));
        const row = evidence.get(name); assert.equal(row?.kind, 'sealed', 'Unknown candidate archive');
        const bytes = await fs.readFile(row.archive); assert.equal(sha256(bytes), row.sha256, 'Candidate changed while serving');
        response.writeHead(200, { 'content-type': 'application/octet-stream' }); response.end(bytes); return;
      }
      const name = decodeURIComponent(url.pathname.slice(1));
      assert.match(name, /^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/i, 'Unexpected registry package route');
      const row = evidence.get(name);
      if (row) {
        let manifest;
        if (row.kind === 'sealed') manifest = { ...row.manifest, dist: { integrity: row.integrity,
          tarball: `http://127.0.0.1:${server.address().port}/archives/${encodeURIComponent(name)}` } };
        else {
          const remote = await fetcher(`https://registry.npmjs.org/${encodeURIComponent(name)}/${encodeURIComponent(row.version)}`, { signal: AbortSignal.timeout(30000), headers: { Accept: 'application/vnd.npm.install-v1+json' } });
          assert.ok(remote.ok, `Immutable reuse metadata unavailable: ${name}`); manifest = JSON.parse(await readRegistryBody(remote, 2 * 1024 * 1024));
          assert.equal(manifest.name, name); assert.equal(manifest.version, row.version); assert.equal(manifest.dist?.integrity, row.integrity);
          assert.equal(new URL(manifest.dist.tarball).origin, 'https://registry.npmjs.org');
        }
        response.writeHead(200, { 'content-type': 'application/json' });
        response.end(JSON.stringify({ name, versions: { [row.version]: manifest }, 'dist-tags': { latest: row.version } })); return;
      }
      // External dependencies are fetched by npm from the official registry only;
      // this proxy never follows arbitrary artifact-provided URLs or credentials.
      const remote = await fetcher(`https://registry.npmjs.org/${encodeURIComponent(name)}`, { signal: AbortSignal.timeout(30000), headers: { Accept: 'application/vnd.npm.install-v1+json' } });
      const bytes = await readRegistryBody(remote);
      if (remote.ok) for (const version of Object.values(JSON.parse(bytes).versions ?? {})) if (version.dist?.tarball) assert.equal(new URL(version.dist.tarball).origin, 'https://registry.npmjs.org', 'External archive origin is not official registry');
      response.writeHead(remote.status, { 'content-type': 'application/json' }); response.end(bytes);
    } catch (error) { response.writeHead(502, { 'content-type': 'application/json' }); response.end(JSON.stringify({ error: error.message })); }
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  return { url: `http://127.0.0.1:${server.address().port}/`, close: () => new Promise(resolve => { server.close(resolve); server.closeAllConnections(); }) };
}
