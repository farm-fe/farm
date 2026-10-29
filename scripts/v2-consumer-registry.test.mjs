import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { gzipSync } from 'node:zlib';
import { candidateNpmEvidence, verifyInstalledNpmLock, serveCandidateRegistry, readRegistryBody, consumerHostAbi } from './v2-consumer-registry.mjs';
import { sha256 } from './release-archive.mjs';
function tar(manifest) {
  const bytes = Buffer.from(JSON.stringify(manifest)), header = Buffer.alloc(512);
  header.write('package/package.json'); header.write(bytes.length.toString(8).padStart(11, '0') + '\0', 124);
  header.fill(32, 148, 156); header[156] = 48;
  header.write([...header].reduce((sum, byte) => sum + byte, 0).toString(8).padStart(6, '0') + '\0 ', 148);
  return gzipSync(Buffer.concat([header, bytes, Buffer.alloc((512 - bytes.length % 512) % 512 + 1024)]));
}
test('candidate registry serves only checked immutable archives and npm lock proves exact installation', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-candidate-registry-'));
  let server;
  try {
    const row = { name: '@farmfe/fixture', version: '2.0.0' }, plan = { npm: { primary: [row], payloads: [] } };
    const bytes = tar(row); await fs.writeFile(path.join(directory, 'fixture.tgz'), bytes);
    const receipt = { planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [{ ...row, file: 'fixture.tgz', sha256: sha256(bytes) }] };
    const evidence = await candidateNpmEvidence(plan, receipt, directory);
    server = await serveCandidateRegistry(evidence, { fetcher: async () => { throw new Error('No external network expected'); } });
    const metadata = await (await fetch(`${server.url}${encodeURIComponent(row.name)}`)).json();
    const archive = Buffer.from(await (await fetch(metadata.versions[row.version].dist.tarball)).arrayBuffer());
    assert.equal(sha256(archive), receipt.records[0].sha256);
    const manifest = { dependencies: { [row.name]: row.version } };
    const lock = { lockfileVersion: 3, packages: { [`node_modules/${row.name}`]: { version: row.version, integrity: evidence.get(row.name).integrity, resolved: metadata.versions[row.version].dist.tarball } } };
    verifyInstalledNpmLock(plan, manifest, lock, evidence, { registry: server.url });
    lock.packages[`node_modules/${row.name}`].integrity = 'sha512-wrong';
    assert.throws(() => verifyInstalledNpmLock(plan, manifest, lock, evidence, { registry: server.url }), /immutable bytes/);
    assert.equal((await fetch(`${server.url}archives/unknown`)).status, 502);
    receipt.records[0].file = '../fixture.tgz';
    await assert.rejects(candidateNpmEvidence(plan, receipt, directory), /Unsafe candidate archive/);
  } finally { await server?.close(); await fs.rm(directory, { recursive: true }); }
});
test('future delta reuses unchanged family only through exact recorded registry integrity', async () => {
  const plan = { npm: { primary: [{ name: 'support', version: '1.0.0' }], payloads: [] }, publication: { npm: { primary: [], payloads: [] }, reuseNpm: [{ name: 'support', version: '1.0.0', integrity: 'sha512-YWJjZA==' }] } };
  const evidence = await candidateNpmEvidence(plan, { planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [] }, '.');
  const lock = { lockfileVersion: 3, packages: { 'node_modules/support': { version: '1.0.0', integrity: 'sha512-YWJjZA==', resolved: 'https://registry.npmjs.org/support/-/support-1.0.0.tgz' } } };
  verifyInstalledNpmLock(plan, { dependencies: { support: '1.0.0' } }, lock, evidence, { registry: 'https://registry.npmjs.org/' });
  lock.packages['node_modules/support'].link = true;
  assert.throws(() => verifyInstalledNpmLock(plan, { dependencies: { support: '1.0.0' } }, lock, evidence, { registry: 'https://registry.npmjs.org/' }), /link/);
  plan.publication.reuseNpm = [];
  await assert.rejects(candidateNpmEvidence(plan, { planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [] }, '.'), /Missing exact family/);
});

test('external registry bodies are bounded while streaming, including absent/false content length', async () => {
  assert.equal((await readRegistryBody(new Response('small'), 5)).toString(), 'small');
  await assert.rejects(readRegistryBody(new Response('large', { headers: { 'content-length': '1000' } }), 8), /too large/);
  await assert.rejects(readRegistryBody(new Response(new ReadableStream({ start(controller) {
    controller.enqueue(new Uint8Array(5)); controller.enqueue(new Uint8Array(5)); controller.close();
  } })), 8), /too large/);
});

test('consumer native ABI uses declared Windows MSVC, Darwin and Linux libc targets', () => {
  const unavailable = () => { throw new Error('Non-Linux must not inspect libc'); };
  for (const arch of ['x64', 'arm64', 'ia32']) assert.equal(consumerHostAbi({ platform: 'win32', arch, report: unavailable }), `win32-${arch}-msvc`);
  for (const arch of ['x64', 'arm64']) {
    assert.equal(consumerHostAbi({ platform: 'darwin', arch, report: unavailable }), `darwin-${arch}`);
    assert.equal(consumerHostAbi({ platform: 'linux', arch, report: () => ({ header: { glibcVersionRuntime: '2.39' } }) }), `linux-${arch}-gnu`);
    assert.equal(consumerHostAbi({ platform: 'linux', arch, report: () => ({ header: {} }) }), `linux-${arch}-musl`);
  }
  for (const options of [{ platform: 'win32', arch: 'arm' }, { platform: 'unknown', arch: 'x64' }, { platform: 'linux', arch: 'x64', report: () => null }, { platform: 'linux', arch: 'x64', report: () => ({ header: { glibcVersionRuntime: 42 } }) }]) assert.throws(() => consumerHostAbi(options));
});
