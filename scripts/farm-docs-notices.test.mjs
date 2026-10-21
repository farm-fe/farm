import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const website = path.join(root, 'website');
const manifest = JSON.parse(fs.readFileSync(path.join(website, 'third-party-notices.manifest.json'), 'utf8'));
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const notice = fs.readFileSync(path.join(website, 'static/THIRD_PARTY_NOTICES.txt'));
assert.equal(sha(notice), manifest.noticeSha256, 'Notice source changed without updating provenance');
assert.equal(new Set(manifest.sources.map(source => `${source.name}@${source.version}`)).size, manifest.packageVersions);
for (const source of manifest.sources) {
  const file = path.resolve(website, source.originalText);
  assert.ok(file.startsWith(path.join(website, 'licenses') + path.sep));
  const original = fs.readFileSync(file);
  assert.equal(sha(original), source.sha256, `${source.name}: original license digest`);
  assert.ok(notice.includes(original), `${source.name}: original license bytes missing from notice`);
}
const css = fs.readFileSync(path.resolve(website, manifest.css.originalText));
assert.equal(sha(css), manifest.css.sha256);
const currentCssLicense = fs.readFileSync(path.resolve(website, manifest.css.source), 'utf8').replace(/\r\n/g, '\n');
assert.equal(sha(currentCssLicense), manifest.css.sourceNormalizedSha256, 'CSS generator source license text changed');
assert.ok(notice.includes(css), 'CSS generator license bytes missing');
const footer = fs.readFileSync(path.join(website, 'src/theme/Footer/index.tsx'), 'utf8');
assert.ok(footer.includes('href="/THIRD_PARTY_NOTICES.txt"'));
if (!process.argv.includes('--source-only')) {
  const output = path.join(website, 'build-farm');
  assert.deepEqual(fs.readFileSync(path.join(output, 'THIRD_PARTY_NOTICES.txt')), notice, 'Published notice differs from source');
  const html = fs.readFileSync(path.join(output, 'index.html'), 'utf8');
  assert.match(html, /href="\/THIRD_PARTY_NOTICES\.txt"/);
}
console.log(`Third-party notices verified: ${manifest.packageVersions} package versions, original license bytes and CSS templates${process.argv.includes('--source-only') ? ' (source only)' : ', output and homepage link'}.`);
