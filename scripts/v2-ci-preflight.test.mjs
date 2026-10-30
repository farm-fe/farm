import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
const read = file => fs.readFileSync(new URL(`../.github/workflows/${file}`, import.meta.url), 'utf8').replaceAll('\r\n', '\n');

test('native matrices wait for cheap spelling, release/pack and three-platform performance contracts', () => {
  const ci = read('ci.yaml');
  const preflight = ci.split('  release-contract-tests:')[1].split('  docs-acceptance:')[0];
  assert.match(preflight, /pnpm exec cspell "\*\*" --gitignore/);
  for (const command of ['test-native-container-node.cjs', 'test-native-artifact-flow.cjs', 'pack-v2-npm-candidates.test.mjs', 'verify-published-v2-consumers.test.mjs']) assert.ok(preflight.includes(command), command);
  for (const job of ['primary-linux', 'primary-macos', 'primary-windows', 'call-rust-build-release']) {
    const block = ci.split(`  ${job}:`)[1].split('\n\n')[0];
    assert.match(block, /needs: \[release-contract-tests, performance-contract-tests\]/);
  }
});

test('primary platform consumers wait only for their own successful producer', () => {
  const ci = read('ci.yaml'), platform = read('ci-primary-platform.yaml'), native = read('rust-build-user.yaml');
  const lanes = [['primary-linux', 'ubuntu-24.04', 'linux-x64-gnu'], ['primary-macos', 'macos-latest', 'darwin-arm64'], ['primary-windows', 'windows-latest', 'win32-x64-msvc']];
  for (const [job, os, abi] of lanes) {
    const block = ci.split(`  ${job}:`)[1].split('\n\n')[0];
    assert.match(block, /uses: \.\/\.github\/workflows\/ci-primary-platform.yaml/);
    assert.ok(block.includes(`os: ${os}`)); assert.ok(block.includes(`abi: ${abi}`));
    assert.ok(native.includes(`inputs.platform == '${abi}'`));
  }
  assert.match(platform, /platform: \$\{\{ inputs.abi \}\}/);
  for (const job of ['examples-test', 'ts-test']) {
    const block = platform.split(`  ${job}:`)[1].split('\n\n')[0];
    assert.match(block, /needs: \[native\]/);
    assert.doesNotMatch(block, /call-rust-build-user|call-rust-build-release|primary-linux|primary-macos|primary-windows/);
  }
  assert.match(native, /group: rust-build-user-.*inputs.platform/);
  assert.match(native, /inputs.platform == 'all'/);
  assert.ok(native.includes("|| '[]'"), 'Unknown platform cannot choose a successful empty execution');
  for (const job of ['call-rust-build-user', 'examples-test', 'ts-test']) {
    const block = ci.split(`  ${job}:`)[1].split('\n\n')[0];
    assert.match(block, /if: always\(\)/);
    assert.match(block, /needs: \[primary-linux, primary-macos, primary-windows\]/);
    assert.ok(block.includes('test "$LINUX_RESULT" = success && test "$MACOS_RESULT" = success && test "$WINDOWS_RESULT" = success'));
  }
});

test('Cargo restores cover all nine targets while only the three primary producers save target caches', () => {
  for (const file of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const source = read(file);
    assert.match(source, /uses: \.\/\.github\/actions\/cargo-source-cache/);
    assert.ok(source.includes('path.join(os.homedir(), \'.cargo\')'));
    assert.ok(source.includes('FARM_CARGO_HOME=${home}'));
    assert.match(source, /node scripts\/v2-docker-cargo-cache.mjs/);
    assert.match(source, /path: .ci-native-cache\/\$\{\{ matrix.settings.abi \}\}.json/);
    assert.doesNotMatch(source, /env.HOME/);
    assert.doesNotMatch(source, /addnab\/docker-run-action|:\/root\/.cargo\/(?:git|registry)/);
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

test('cache writers are limited without replacing the Docs release reference or final nine-ABI assembly', () => {
  const user = read('rust-build-user.yaml'), release = read('rust-build-release.yaml'), docs = read('farm-docs.yaml');
  assert.match(user, /save-if: true/);
  for (const source of [release, docs]) {
    assert.doesNotMatch(source, /Swatinem\/rust-cache|writer: 'true'/);
    assert.match(source, /uses: \.\/\.github\/actions\/cargo-source-cache/);
  }
  assert.doesNotMatch(docs, /cache: pnpm/);
  assert.match(docs, /pnpm --dir packages\/core run build:rs/);
  assert.match(docs, /ciProfileCompatibility:'not-yet-validated'/);
  const ci = read('ci.yaml');
  assert.match(ci.split('  docs-acceptance:')[1].split('  release-candidate-seal:')[0], /needs: \[release-contract-tests, performance-contract-tests\]/);
  const assembly = ci.split('  check-native-candidate-assembly:')[1].split('  examples-test:')[0];
  assert.match(assembly, /needs: \[call-rust-build-user, call-rust-build-release\]/);
});

test('source archives and native target caches have disjoint ownership and stable compatible keys', () => {
  const action = fs.readFileSync(new URL('../.github/actions/cargo-source-cache/action.yml', import.meta.url), 'utf8');
  for (const member of ['~/.cargo/registry/cache', '~/.cargo/registry/index', '~/.cargo/git/db']) assert.equal(action.split(member).length - 1, 2);
  const paths = action.split('\n').map(line => line.trim()).filter(line => line.startsWith('~/.cargo/'));
  assert.deepEqual([...new Set(paths)].sort(), ['~/.cargo/git/db', '~/.cargo/registry/cache', '~/.cargo/registry/index']);
  assert.doesNotMatch(action, /run_id|github\.sha|\*\*\/Cargo\.toml/);
  for (const input of ['Cargo.lock', 'crates/*/Cargo.toml', 'rust-plugins/*/Cargo.toml', 'rust-ecosystem/*/Cargo.toml', 'packages/create-farm/Cargo.toml']) assert.ok(action.includes(input), input);
  assert.match(action, /inputs.writer == 'true'/); assert.match(action, /actions\/cache\/restore@v4/);
  assert.match(action, /runner.os/); assert.match(action, /runner.arch/); assert.match(action, /inputs.generation/);
  const user = read('rust-build-user.yaml');
  assert.match(user, /CARGO_HOME: \$\{\{ runner.temp \}\}\/farm-target-cache-home/);
  assert.match(user, /cmd-format: node scripts\/cargo-cache-command.cjs \{0\}/);
  assert.match(user, /cache-bin: false/); assert.match(user, /cache-on-failure: false/);
  assert.match(user, /prefix-key: rust-target-v1-\$\{\{ inputs.profile \}\}-\$\{\{ matrix.settings.abi \}\}/);
  assert.ok(user.includes('uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0'));
  assert.match(user, /env-vars: FARM_NATIVE_CACHE_CONTEXT/);
  assert.match(user, /docker image inspect --format/);
  for (const file of ['rust-build-release.yaml', 'farm-docs.yaml', 'rust-test.yaml']) assert.doesNotMatch(read(file), /writer: 'true'/);
});

test('Unit preserves its existing ci-test profile and source fetch retries do not wrap compiler failures', () => {
  const unit = read('rust-test.yaml'), docs = read('farm-docs.yaml');
  assert.match(unit, /cargo test --locked --profile ci-test --tests/);
  assert.match(unit, /cargo check --locked --color always --all --all-targets/);
  assert.doesNotMatch(unit, /Swatinem\/rust-cache|cargo test.*\|\|/);
  assert.equal(unit.split('run: cargo fetch --locked').length - 1, 2);
  for (const source of [unit, docs]) {
    assert.match(source, /timeout-minutes: 10/);
    assert.match(source, /CARGO_NET_RETRY: '3'/); assert.match(source, /CARGO_HTTP_TIMEOUT: '60'/);
  }
});

test('validated metadata precedes cache registration and is checked again before successful post-save', () => {
  const user = read('rust-build-user.yaml');
  const capture = user.indexOf('run: node scripts/cargo-cache-command.cjs --snapshot');
  const action = user.indexOf('uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0');
  const verify = user.indexOf('run: node scripts/cargo-cache-command.cjs --verify-snapshot');
  assert.ok(capture >= 0 && action > capture && verify > action && user.indexOf('name: Upload Core') > verify);
  assert.match(user, /cache-on-failure: false/);
  assert.equal(user.split('FARM_CARGO_METADATA: ${{ runner.temp }}/farm-cargo-metadata.json').length - 1, 3);
});
