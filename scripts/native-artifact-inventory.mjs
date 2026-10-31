import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export const nativeTargets = Object.freeze({
  'x86_64-unknown-linux-gnu': 'linux-x64-gnu',
  'x86_64-unknown-linux-musl': 'linux-x64-musl',
  'aarch64-unknown-linux-gnu': 'linux-arm64-gnu',
  'aarch64-unknown-linux-musl': 'linux-arm64-musl',
  'x86_64-apple-darwin': 'darwin-x64',
  'aarch64-apple-darwin': 'darwin-arm64',
  'x86_64-pc-windows-msvc': 'win32-x64-msvc',
  'i686-pc-windows-msvc': 'win32-ia32-msvc',
  'aarch64-pc-windows-msvc': 'win32-arm64-msvc'
});
// CI input discovery, deliberately independent of the one-off release inventory.
export function nativeBuildSupport(root) {
  const recipe = fs.readFileSync(path.join(root, 'scripts/ci-build-rust-artifacts.sh'), 'utf8');
  const regularPlugins = recipe.match(/^PLUGINS="([^"]+)"/m)?.[1].split(' ');
  const docs = recipe.match(/^\s+((?:win32|linux|darwin)[^\n)]+)\)\r?\n\s+PLUGINS="\$PLUGINS ([^"]+)"/m);
  assert.ok(regularPlugins && docs, 'Native build support recipe changed; review discovery');
  const abis = [...new Set(['rust-build-user.yaml', 'rust-build-release.yaml'].flatMap(file => [...fs.readFileSync(path.join(root, '.github/workflows', file), 'utf8').matchAll(/^\s+abi:\s*([a-z0-9-]+)\s*$/gm)].map(match => match[1])))];
  assert.ok(abis.length && abis.every(abi => Object.values(nativeTargets).includes(abi)), 'Unknown CI native support matrix');
  return { abis, regularPlugins, docsAbis: docs[1].split('|'), docsPlugins: docs[2].split(' ') };
}
export function nativeArtifactInventory(root, support = nativeBuildSupport(root)) {
  const primary = [], payloads = [];
  for (const group of ['packages', 'rust-plugins']) {
    const directory = path.join(root, group);
    if (!fs.existsSync(directory)) continue;
    for (const entry of fs.readdirSync(directory).sort()) {
      const manifestPath = `${group}/${entry}/package.json`;
      if (!fs.existsSync(path.join(root, manifestPath))) continue;
      const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath)));
      if (manifest.private || !manifest.napi) continue;
      if (manifest.napi.targets) assert.ok(Array.isArray(manifest.napi.targets) && new Set(manifest.napi.targets).size === manifest.napi.targets.length, 'Duplicate native target declaration');
      primary.push({ name: manifest.name, version: manifest.version, manifestPath });
      // Local napi defaults can be narrower than the actual CI cross matrix.
      let platforms = support.abis;
      if (group === 'rust-plugins') {
        if (support.docsPlugins.includes(entry)) platforms = support.docsAbis;
        else assert.ok(support.regularPlugins.includes(entry), `Native plugin is absent from build recipe: ${entry}`);
      }
      assert.ok(platforms.length, 'Native platform manifests are missing');
      for (const platform of platforms) {
        assert.ok(Object.values(nativeTargets).includes(platform), `Unsupported native platform package: ${platform}`);
        const file = `${group}/${entry}/npm/${platform}/package.json`;
        const payload = JSON.parse(fs.readFileSync(path.join(root, file)));
        assert.equal(payload.name, `${manifest.name}-${platform}`, `Native payload name mismatch: ${file}`);
        assert.equal(payload.version, manifest.version, `Native payload version mismatch: ${file}`);
        const [os, cpu, libc] = platform.split('-');
        assert.deepEqual(payload.os, [os], `Native payload OS mismatch: ${file}`);
        assert.deepEqual(payload.cpu, [cpu], `Native payload CPU mismatch: ${file}`);
        if (os === 'linux') assert.deepEqual(payload.libc, [libc === 'gnu' ? 'glibc' : 'musl'], `Native payload libc mismatch: ${file}`);
        assert.ok(Object.hasOwn(manifest.optionalDependencies ?? {}, payload.name), `Native payload optional dependency missing: ${file}`);
        assert.ok([manifest.version, 'workspace:*'].includes(manifest.optionalDependencies[payload.name]), `Native payload optional dependency version mismatch: ${file}`);
        payloads.push({ name: payload.name, version: manifest.version, owner: manifest.name, platform, manifestPath: file });
      }
    }
  }
  assert.ok(payloads.length, 'No native build manifests discovered');
  assert.equal(new Set(payloads.map(row => row.name)).size, payloads.length, 'Duplicate native package');
  return { npm: { primary, payloads } };
}
