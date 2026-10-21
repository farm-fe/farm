import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {verifyDocsPackageDirectory} from './farm-docs-package-contract.mjs';

// Receipts must come from the recorded extractor execution, not hand-authored JSON.
// This checks drift and joins contracts; it does not authenticate arbitrary receipts.
export async function validateDocsExtraction(receipt,{archive,sha256,...contractOptions}={}) {
  assert.equal(receipt?.success,true,'Successful extraction receipt is required');
  assert(typeof sha256==='string'&&/^[a-f0-9]{64}$/i.test(sha256),'Expected archive digest must be SHA256');
  const actualArchive=await fs.realpath(archive),receiptArchive=await fs.realpath(receipt.archive);
  assert.equal(path.relative(actualArchive,receiptArchive),'','Extraction receipt archive path differs');
  const expected=sha256.toLowerCase();
  assert.equal(receipt.sha256,expected,'Extraction receipt archive digest differs from pack record');
  const bytes=await fs.readFile(actualArchive);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),expected,'Current archive digest differs from pack record');
  const directoryReceipt=await verifyDocsPackageDirectory(receipt.destination,contractOptions);
  const fields=files=>files.map(file=>({path:file.path,bytes:file.bytes,sha256:file.sha256})).sort((a,b)=>a.path.localeCompare(b.path));
  assert(Array.isArray(receipt.files),'Extraction receipt files are required');
  assert.deepEqual(fields(directoryReceipt.files),fields(receipt.files),'Extracted files differ from receipt files/digests');
  const manifest=JSON.parse(await fs.readFile(path.join(receipt.destination,'package.json'),'utf8'));
  assert.deepEqual(manifest,receipt.manifest,'Extracted manifest differs from receipt manifest');
  return {manifest,archive:actualArchive,sha256:expected,directoryReceipt};
}
