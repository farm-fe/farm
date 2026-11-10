import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';

test('native routing fixture is isolated from CI and release producer controls', () => {
  for (const environment of [{}, {FARM_RELEASE_NATIVE_OWNERS: '["@farmfe/core","@farmfe/plugin-react"]', FARM_BUILD_PROFILE: 'release', FARM_NATIVE_PREPARE_ONLY:'true', FARM_NATIVE_PROOF_MODE:'shadow', FARM_FORMAL_RESOLUTION_HASH:'ambient-selection', FARM_BUILD_TARGET:'ambient-target', FARM_BUILD_ZIG:'true'}]) {
    const result=spawnSync(process.execPath,['scripts/test-native-artifact-flow.cjs'],{env:{...process.env,...environment},encoding:'utf8',timeout:60000});
    assert.equal(result.status,0,result.stderr+result.stdout);
    const report=JSON.parse(result.stdout.trim());
    assert.equal(report.targets,9);
    assert.equal(report.missingArtifactRejected,true);
    assert.equal(report.releaseScopeVerified,true);
    assert.equal(report.probeFailuresRejected,true);
  }
});
