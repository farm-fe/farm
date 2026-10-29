import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
const read = file => fs.readFileSync(new URL(`../.github/workflows/${file}`, import.meta.url), 'utf8').replaceAll('\r\n', '\n');

test('native matrices wait for cheap spelling, release/pack and three-platform performance contracts', () => {
  const ci = read('ci.yaml');
  const preflight = ci.split('  release-contract-tests:')[1].split('  docs-acceptance:')[0];
  assert.match(preflight, /pnpm exec cspell "\*\*" --gitignore/);
  for (const command of ['test-native-container-node.cjs', 'test-native-artifact-flow.cjs', 'pack-v2-npm-candidates.test.mjs', 'verify-published-v2-consumers.test.mjs']) assert.ok(preflight.includes(command), command);
  for (const job of ['call-rust-build-user', 'call-rust-build-release']) {
    const block = ci.split(`  ${job}:`)[1].split('\n\n')[0];
    assert.match(block, /needs: \[release-contract-tests, performance-contract-tests\]/);
  }
});

test('Cargo caches include the six release targets and actual Docker host mounts without caching uploaded payloads', () => {
  for (const file of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const source = read(file);
    assert.match(source, /uses: Swatinem\/rust-cache@v2/);
    assert.match(source, /workspaces: ". -> target"/);
    assert.match(source, /FARM_CARGO_HOME=%s\/.cargo/);
    assert.match(source, /env.FARM_CARGO_HOME \}\}\/registry:\/root\/.cargo\/registry/);
    assert.doesNotMatch(source, /env.HOME/);
    assert.ok(source.includes('-v ${{ github.workspace }}:/build'));
    assert.ok(source.includes('scripts/ci-build-rust-artifacts.sh'));
    assert.ok(source.includes('name: ${{ github.sha }}-${{ matrix.settings.abi }}'));
  }
  const release = read('rust-build-release.yaml');
  for (const target of ['linux-x64-musl', 'darwin-x64', 'win32-ia32-msvc', 'win32-arm64-msvc', 'linux-arm64-musl', 'linux-arm64-gnu']) assert.ok(release.includes(`abi: ${target}`));
});

test('three independent candidate consumer jobs install the exact current seal without workspace dependencies', () => {
  const source = read('v2-candidate-seal.yaml').split('  consumers:')[1];
  assert.ok(source);
  assert.match(source, /needs: \[seal\]/);
  assert.match(source, /os: \[ubuntu-24.04, macos-latest, windows-latest\]/);
  assert.ok(source.includes('name: v2-release-seal-${{ github.sha }}'));
  assert.match(source, /node scripts\/verify-v2-npm-consumer.mjs release-candidate candidate-consumer-results/);
  assert.doesNotMatch(source, /pnpm install|cargo |NPM_TOKEN|secrets:/);
});
