import test from 'node:test';
import assert from 'node:assert/strict';
import { checkPublishedRustClosure } from './check-published-rust-closure.mjs';
const pkg = { name: 'farmfe_core', version: '2.2.0', sha256: 'a'.repeat(64) };
test('every selected Rust registry version must be present, unyanked and byte-identical', async () => {
  const metadata = { num: pkg.version, yanked: false, checksum: pkg.sha256 };
  const fetchImpl = async () => Response.json({ version: metadata });
  assert.equal((await checkPublishedRustClosure([pkg], { fetchImpl })).count, 1);
  for (const changed of [{ ...metadata, checksum: 'b'.repeat(64) }, { ...metadata, yanked: true }, { ...metadata, num: '2.1.0' }]) await assert.rejects(checkPublishedRustClosure([pkg], { fetchImpl: async () => Response.json({ version: changed }) }));
  await assert.rejects(checkPublishedRustClosure([pkg], { fetchImpl: async () => new Response('', { status: 404 }) }), /lookup failed/);
});
