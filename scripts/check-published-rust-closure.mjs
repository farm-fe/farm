import assert from 'node:assert/strict';
export async function checkPublishedRustClosure(packages, { fetchImpl = fetch } = {}) {
  assert.ok(packages.length && new Set(packages.map(pkg => pkg.name)).size === packages.length, 'Rust registry allowlist is missing or duplicated');
  const records = [];
  for (const pkg of packages) {
    const url = `https://crates.io/api/v1/crates/${encodeURIComponent(pkg.name)}/${encodeURIComponent(pkg.version)}`;
    const response = await fetchImpl(url, { headers: { 'User-Agent': 'Farm-stable-SDK-check' }, signal: AbortSignal.timeout(30000) });
    assert.ok(response.ok, `Published Rust SDK lookup failed: ${pkg.name} HTTP ${response.status}`);
    const version = (await response.json()).version;
    assert.equal(version?.num, pkg.version, `Published Rust SDK version differs: ${pkg.name}`);
    assert.equal(version.yanked, false, `Published Rust SDK version is yanked: ${pkg.name}`);
    assert.equal(version.checksum, pkg.sha256, `Published Rust SDK bytes differ: ${pkg.name}`);
    records.push({ name: pkg.name, version: pkg.version, sha256: pkg.sha256 });
  }
  return { verified: true, count: records.length, records };
}
