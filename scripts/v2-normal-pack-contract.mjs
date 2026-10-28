import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';

export function verifyLicenseComponents(directory, manifest, repositoryRoot = directory) {
  const file = path.join(directory, 'license-components.json');
  if (!fs.existsSync(file)) return;
  const record = JSON.parse(fs.readFileSync(file));
  assert.equal(record.schemaVersion, 1);
  assert.equal(record.declaration, manifest.license, 'Component SPDX declaration differs from package');
  const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
  for (const component of record.components) {
    assert.equal(component.status, 'verified', `Unresolved license component ${component.name}: ${component.reason}`);
    const licenses = component.licenseFiles ?? [{ file: component.licenseFile, sha256: component.licenseSha256, normalization: component.licenseNormalization }];
    assert.ok(licenses.length, 'Original license material required');
    for (const license of licenses) {
      assert.ok(license.file && !path.isAbsolute(license.file) && !license.file.split('/').includes('..'));
      const bytes = fs.readFileSync(path.join(directory, license.file));
      assert.equal(digest(license.normalization === 'lf' ? bytes.toString('utf8').replaceAll('\r\n', '\n') : bytes), license.sha256, `Original license bytes changed: ${component.name}`);
    }
    for (const source of component.sourceFiles ?? []) {
      const target = path.resolve(directory, source.file), relative = path.relative(repositoryRoot, target);
      assert.ok(!relative.startsWith('..') && !path.isAbsolute(relative), 'Component source escapes repository');
      assert.equal(digest(fs.readFileSync(target, 'utf8').replaceAll('\r\n', '\n')), source.sha256, `Component source scope changed: ${source.file}`);
    }
  }
}

export function assertStableManifest(manifest, ownNames = new Set()) {
  assert(/^\d+\.\d+\.\d+$/.test(manifest.version) && manifest.version !== '0.0.0', manifest.name + ': stable version required');
  for (const section of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
    for (const [name, value] of Object.entries(manifest[section] ?? {})) {
      assert.equal(typeof value, 'string', name + ': dependency must be a string');
      assert(!value.startsWith('workspace:'), name + ': unresolved workspace reference');
      if (name.startsWith('@farmfe/') || ownNames.has(name)) {
        assert(!/\d\.\d+\.\d+-/.test(value), name + ': prerelease self dependency');
        assert(!/^(?:file|link):/.test(value), name + ': local self dependency');
        assert(!/(?:^|[~^\s=])0\.0\.0(?:$|[\s|])/.test(value), name + ': placeholder self dependency');
      }
    }
  }
}

function publicTargets(value) {
  if (value == null) return [];
  if (typeof value === 'string') return [value];
  assert(typeof value === 'object'); return Object.values(value).flatMap(publicTargets);
}

export function verifyPublicEntries(manifest, files) {
  const names = files.map(row => row.path);
  const targets = [...new Set(['main', 'module', 'types', 'typings', 'bin', 'exports'].flatMap(field => publicTargets(manifest[field])))];
  assert(targets.length, manifest.name + ': no public entry');
  for (const target of targets) {
    assert(!path.posix.isAbsolute(target) && !path.win32.isAbsolute(target) && !target.includes('\\') && !target.split('/').includes('..'), target);
    const name = target.replace(/^\.\//, '');
    const escaped = name.split('*').map(part => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('.*');
    const pattern = new RegExp('^' + escaped + '$');
    assert(names.some(file => pattern.test(file)), manifest.name + ': unpublished entry ' + target);
  }
}

export function verifyArchiveMemberSet(files, requiredNames, allowedAdditional = []) {
  const actual = files.map(row => row.path), names = new Set(actual);
  assert.equal(actual.length, names.size, 'Duplicate archive members');
  const required = new Set(requiredNames), allowed = new Set([...required, ...allowedAdditional]);
  const missing = [...required].filter(name => !names.has(name));
  const unexpected = actual.filter(name => !allowed.has(name));
  assert.deepEqual(missing, [], 'Dropped package payload: ' + missing.join(', '));
  assert.deepEqual(unexpected, [], 'Unreviewed package payload: ' + unexpected.join(', '));
  return actual.filter(name => !required.has(name)).sort();
}

export function verifyWorkspaceConversion(original, packed, versions) {
  for (const section of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
    for (const [name, reference] of Object.entries(original[section] ?? {})) {
      if (!reference.startsWith('workspace:')) continue;
      assert(versions.has(name), name + ': workspace dependency outside current public build');
      const selector = reference.slice('workspace:'.length);
      assert(['*', '^'].includes(selector), name + ': review new workspace selector ' + selector);
      const expected = (selector === '^' ? '^' : '') + versions.get(name);
      assert.equal(packed[section]?.[name], expected, name + ': pack used another installed workspace version');
    }
  }
}
