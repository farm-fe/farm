// Mocked bootstrap routing/failure tests; this is not native Linux/container proof.
// cspell:ignore libstdc
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const fixture = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-container-node-'));
const unix = p => process.platform === 'win32' ? '/' + p[0].toLowerCase() + p.slice(2).replaceAll('\\', '/') : p;
const bash = process.platform === 'win32' ? 'C:/Program Files/Git/bin/bash.exe' : 'bash';
const trace = path.join(fixture, 'trace');
const bin = path.join(fixture, 'old-bin');
fs.mkdirSync(bin);
fs.mkdirSync(path.join(fixture, 'temp'));
const wrapper = path.join(fixture, 'new-node');
fs.writeFileSync(path.join(bin, 'node'), '#!/usr/bin/env sh\necho v20.18.0\n', {mode: 0o755});
fs.writeFileSync(wrapper, `#!/usr/bin/env sh
if [ "$*" = './scripts/check-native-build-node.mjs' ]; then
  printf 'guard\\n' >> "$TEST_TRACE"
  if [ "$TEST_MODE" = engine ]; then exit 23; fi
fi
exec '${unix(process.execPath)}' "$@"
`, {mode: 0o755});
const probe = path.join(fixture, 'probe.sh');
fs.writeFileSync(probe, `set -eu
uname() { if [ "$1" = -s ]; then printf 'Linux\\n'; elif [ "$TEST_MODE" = arch ]; then printf 'aarch64\\n'; else printf 'x86_64\\n'; fi; }
apk() { printf 'apk|%s\\n' "$*" >> "$TEST_TRACE"; }
curl() {
  printf 'curl|%s\\n' "$7" >> "$TEST_TRACE"
  test "$5" = --output
  test "$7" = "$TEST_URL"
  if [ "$TEST_MODE" = network ]; then return 22; fi
  printf 'fixture archive\\n' > "$6"
}
sha256sum() {
  if [ "$TEST_MODE" = checksum ]; then command sha256sum "$@"; return; fi
  input=$(cat)
  test "$input" = "$TEST_SHA  $TEST_ARCHIVE"
  printf 'checksum\\n' >> "$TEST_TRACE"
}
tar() {
  printf 'tar\\n' >> "$TEST_TRACE"
  test "$1" = -xJf
  test "$3" = -C
  mkdir "$4/bin"
  cp "$TEST_WRAPPER" "$4/bin/node"
  chmod +x "$4/bin/node"
}
. '${unix(path.join(root, 'scripts/setup-native-container-node.sh'))}'
printf 'after|%s\\n' "$(command -v node)" >> "$TEST_TRACE"
`);
const variants = [
  ['linux-x64-gnu', 'linux-x64', 'https://nodejs.org/dist/v22.20.0', '00bbd05e306ea68b6e13e17360d0e2f680b493ef95f2fea1c4296ff7437530bc'],
  ['linux-x64-musl', 'linux-x64-musl', 'https://unofficial-builds.nodejs.org/download/release/v22.20.0', '119817f0a2cb86e9b8f058a9fc0a9fcb1b56a8a6da7c6ebef64398328ee52d82']
];
const results = [];
try {
  cp.execFileSync(bash, ['-n', path.join(root, 'scripts/setup-native-container-node.sh')]);
  for (const [abi, variant, server, sha] of variants) {
    for (const mode of ['success', 'checksum', 'network', 'engine', 'arch']) {
      fs.writeFileSync(trace, '');
      const archive = `node-v22.20.0-${variant}.tar.xz`;
      const env = {...process.env, FARM_BUILD_ABI: abi, TEST_MODE: mode, TEST_TRACE: unix(trace), TEST_WRAPPER: unix(wrapper), TEST_URL: `${server}/${archive}`, TEST_SHA: sha, TEST_ARCHIVE: archive, TMPDIR: unix(path.join(fixture, 'temp'))};
      for (const key of Object.keys(env)) if (key.toLowerCase() === 'path') delete env[key];
      env.PATH = unix(bin) + ':' + (process.platform === 'win32' ? '/usr/bin:/bin:' : '') + process.env.PATH;
      const run = cp.spawnSync(bash, [unix(probe)], {cwd: root, env, encoding: 'utf8', windowsHide: true, timeout: 30000});
      const calls = fs.readFileSync(trace, 'utf8');
      if (mode === 'success') {
        assert.equal(run.status, 0, run.stderr + run.stdout);
        assert.match(run.stdout, /"success":true/);
        assert.match(calls, /guard\nafter\|.*\/runtime\/bin\/node/);
      } else {
        assert.notEqual(run.status, 0, mode);
        assert.ok(!calls.includes('after|'));
        if (mode !== 'engine') assert.ok(!calls.includes('guard'));
        if (['checksum', 'network', 'arch'].includes(mode)) assert.ok(!/^tar$/m.test(calls));
        if (mode === 'arch') assert.ok(!calls.includes('curl'));
      }
      if (mode === 'success' && abi.endsWith('musl')) assert.match(calls, /apk\|add --no-cache curl xz libstdc\+\+/);
      results.push({abi, mode, exit: run.status});
    }
  }
  for (const name of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const workflow = fs.readFileSync(path.join(root, '.github/workflows', name), 'utf8').replaceAll('\r\n', '\n');
    const docker = workflow.split('      - name: Build in docker\n')[1]?.split('      - name: Build\n')[0];
    assert.ok(docker, 'Docker step must exist');
    assert.match(docker, /          run: \|\n            \. \.\/scripts\/setup-native-container-node\.sh\n            \.\/scripts\/ci-build-rust-artifacts\.sh\n$/);
  }
  console.log(JSON.stringify({status: 'passed', cases: results, scope: 'Mocked shell routing and failures with real installed engine guard; actual GNU/musl execution remains a CI gate.'}));
} finally {
  assert.equal(path.dirname(fixture), os.tmpdir());
  assert.ok(path.basename(fixture).startsWith('farm-container-node-'));
  fs.rmSync(fixture, {recursive: true, force: true});
}
