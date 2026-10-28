import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { assertNativeAssembly, assertSourceLicense } from './pack-v2-npm-candidates.mjs';
test('native producer rejects absent, duplicate or different-platform native cohorts', () => {
  const plan = { npm: { payloads: [{ name: 'core-linux', version: '2.0.0', platform: 'linux-x64-gnu' }] } };
  const assembly = { sourceCommit: 'a'.repeat(40), count: 1, records: [{ ...plan.npm.payloads[0], artifact: 'exact-artifact' }] };
  const artifactIndex = [{ name: 'exact-artifact', id: 1 }];
  assertNativeAssembly(plan, assembly, artifactIndex, assembly.sourceCommit);
  assert.throws(() => assertNativeAssembly(plan, assembly, [], assembly.sourceCommit));
  assert.throws(() => assertNativeAssembly(plan, assembly, [...artifactIndex, ...artifactIndex], assembly.sourceCommit));
  assert.throws(() => assertNativeAssembly(plan, assembly, artifactIndex, 'b'.repeat(40)));
});
test('source legal gate does not silently relabel ISC or borrow a root license', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-license-gate-'));
  try {
    assert.throws(() => assertSourceLicense(root, { name: 'generator', license: 'ISC' }));
    fs.writeFileSync(path.join(root, 'LICENSE'), 'MIT License\nPermission is hereby granted, free of charge');
    assertSourceLicense(root, { name: 'core', license: 'MIT' });
    assert.throws(() => assertSourceLicense(root, { name: 'generator', license: 'ISC' }), /ISC declaration differs/);
    assert.throws(() => assertSourceLicense(root, { name: 'unlabeled' }), /Missing license declaration/);
  } finally { fs.rmSync(root, { recursive: true }); }
});
