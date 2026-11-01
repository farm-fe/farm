// cspell:words tomllib
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { parseDocument } from 'yaml';
import semver from 'semver';

const hash = value => crypto.createHash('sha256').update(value).digest('hex');
const equal = (a, b) => assert.deepEqual(a, b);
const clone = value => structuredClone(value);
const dependencyFields = ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies'];
const cargoFields = ['dependencies', 'build-dependencies', 'dev-dependencies'];
const yaml = text => {
  const document = parseDocument(text, { uniqueKeys: true });
  assert.equal(document.errors.length, 0, 'Invalid or duplicate YAML keys');
  return document.toJS({ maxAliasCount: 0 });
};
const toml = text => JSON.parse(execFileSync(process.env.FARM_GRAPH_PYTHON ?? 'python3',
  ['-c', 'import json,sys,tomllib;print(json.dumps(tomllib.loads(sys.stdin.read())))'],
  { input: text, encoding: 'utf8', timeout: 10000, maxBuffer: 8 * 1024 ** 2 }));

// Evidence is supplied by a separate trusted Actions API adapter. Artifact
// assertions alone, event actor names and branch/title conventions grant nothing.
export function verifyVersionProducer({ pr, repository, run, job, artifact, receipt, receiptBytes, archiveBytes }) {
  assert.equal(repository.default_branch, 'main');
  assert.equal(pr.base.repo.id, repository.id); assert.equal(pr.head.repo.id, repository.id);
  assert.equal(pr.base.ref, 'main'); assert.equal(pr.user.type, 'Bot');
  assert.equal(pr.user.id, 41898282); assert.equal(pr.user.login, 'github-actions[bot]');
  assert.equal(run.repository.id, repository.id); assert.equal(run.head_repository.id, repository.id);
  assert.equal(run.head_branch, 'main'); assert.equal(run.path, '.github/workflows/release.yaml');
  assert.ok(['push', 'workflow_run', 'workflow_dispatch'].includes(run.event));
  assert.equal(run.status, 'completed'); assert.equal(run.conclusion, 'success');
  assert.equal(job.run_id, run.id); assert.equal(job.run_attempt, run.run_attempt);
  assert.equal(job.name, 'Prepare reviewed version PR');
  assert.equal(job.status, 'completed'); assert.equal(job.conclusion, 'success');
  assert.ok(job.steps.some(step => step.name === 'Upload version-only provenance' && step.conclusion === 'success'));
  assert.equal(artifact.workflow_run.id, run.id); assert.equal(artifact.expired, false);
  assert.equal(artifact.digest, 'sha256:' + hash(archiveBytes));
  assert.equal(receipt.kind, 'changesets-version-preparation');
  assert.equal(run.head_sha,receipt.baseSHA,'Version producer source differs from preparation base');
  for (const [key, expected] of Object.entries({ repositoryId: repository.id, pr: pr.number,
    baseSHA: pr.base.sha, headSHA: pr.head.sha, runId: run.id, attempt: run.run_attempt })) equal(receipt[key], expected);
  assert.match(receipt.headTree, /^[a-f0-9]{40}$/);
  assert.equal(receiptBytes.toString(), JSON.stringify(receipt) + '\n');
  return receipt;
}

function synchronizeRange(value, previous, next) {
  if (previous === next) return value;
  for (const prefix of ['', '=', '^', '~', 'workspace:', 'workspace:^', 'workspace:~']) {
    if (value === prefix + previous) return prefix + next;
  }
  return value;
}

function packageMaps(before, after, suffix, parse) {
  const isCargo = suffix.endsWith('Cargo.toml');
  const inherited = isCargo ? [before,after].map(files=>files['Cargo.toml'] ? parse(files['Cargo.toml']).workspace?.package?.version : undefined) : [];
  const files = Object.keys(before).filter(file => file.endsWith(suffix));
  const result = new Map();
  for (const file of files) {
    if (!(file in after)) continue;
    const a = parse(before[file]), b = parse(after[file]);
    const old = isCargo ? a.package : a;
    const current = isCargo ? b.package : b;
    if (old?.private === true) continue;
    const inheritedVersion = isCargo && old?.version?.workspace === true;
    const oldVersion = inheritedVersion ? inherited[0] : old?.version;
    const newVersion = inheritedVersion && current?.version?.workspace === true ? inherited[1] : current?.version;
    if (!old?.name || typeof oldVersion !== 'string' || typeof newVersion !== 'string') continue;
    // Duplicate names cannot establish an unambiguous local dependency identity.
    assert.ok(!result.has(old.name), 'Ambiguous version owner');
    assert.equal(current.name, old.name);
    if (oldVersion !== newVersion) assert.ok(semver.valid(newVersion) && semver.gt(newVersion, oldVersion), 'Version must advance');
    result.set(old.name, { file, before: oldVersion, after: newVersion, inheritedVersion });
  }
  return result;
}

function npmManifest(old, current, locals) {
  const expected = clone(old);
  const owner = locals.get(old.name);
  if (owner) expected.version = owner.after;
  for (const field of dependencyFields) for (const [name, version] of Object.entries(expected[field] ?? {})) {
    const target = locals.get(name);
    if (target) expected[field][name] = synchronizeRange(version, target.before, target.after);
  }
  equal(expected, current);
}

function cargoManifest(old, current, locals, fixture = false) {
  const expected = clone(old), owner = locals.get(old.package?.name);
  if (owner && !owner.inheritedVersion) expected.package.version = owner.after;
  if (expected.workspace?.package?.version !== current.workspace?.package?.version) {
    const inherited = [...locals.values()].filter(row=>row.inheritedVersion);
    assert.ok(inherited.length && inherited.every(row=>row.before===expected.workspace.package.version && row.after===current.workspace?.package?.version), 'Unbound workspace version');
    expected.workspace.package.version = current.workspace.package.version;
  }
  for (const table of [expected, expected.workspace ?? {}, ...Object.values(expected.target ?? {})]) {
    for (const field of cargoFields) for (const [alias, declaration] of Object.entries(table[field] ?? {})) {
      if (fixture && typeof declaration === 'string' && locals.has(alias)) {
        const target = locals.get(alias);table[field][alias] = synchronizeRange(declaration,target.before,target.after);continue;
      }
      if (!declaration || typeof declaration !== 'object') continue;
      const target = locals.get(declaration.package ?? alias);
      // Only explicit local path dependencies can change constraints. An
      // external package with the same name does not become local by spelling.
      if (target && (declaration.path || fixture) && declaration.version) {
        declaration.version = synchronizeRange(declaration.version, target.before, target.after);
      }
    }
  }
  equal(expected, current);
}

function pnpmLock(old, current, locals) {
  const expected = clone(old);
  for (const [directory, importer] of Object.entries(expected.importers ?? {})) {
    for (const field of dependencyFields) for (const [name, declaration] of Object.entries(importer[field] ?? {})) {
      const target = locals.get(name);
      if (!target) continue;
      // Workspace links retain the same target. Any resolved external package,
      // snapshot/peer graph, integrity, settings or patches must stay identical.
      if (typeof declaration.version === 'string' && declaration.version.startsWith('link:')
        && path.posix.normalize(path.posix.join(directory, declaration.version.slice(5))) === path.posix.dirname(target.file)) {
        declaration.specifier = synchronizeRange(declaration.specifier, target.before, target.after);
      }
    }
  }
  equal(expected, current);
}

function cargoLock(old, current, locals) {
  const expected = clone(old);
  for (const pkg of expected.package ?? []) {
    const owner = locals.get(pkg.name);
    if (!pkg.source && owner && pkg.version === owner.before) pkg.version = owner.after;
    pkg.dependencies = pkg.dependencies?.map(value => {
      for (const [name, target] of locals) if (value === name + ' ' + target.before) {
        assert.ok(!(old.package??[]).some(row=>row.name===name && row.version===target.before && row.source),'Ambiguous local/external Cargo lock identity');
        return name + ' ' + target.after;
      }
      return value;
    });
    if (pkg.dependencies === undefined) delete pkg.dependencies;
  }
  equal(expected, current);
}

export function classifyVersionDiff({ before, after, modesBefore, modesAfter, receipt }) {
  try {
    const npm = packageMaps(before, after, '/package.json', JSON.parse);
    const cargo = packageMaps(before, after, '/Cargo.toml', toml);
    const changed = [...new Set([...Object.keys(before), ...Object.keys(after)])]
      .filter(file => before[file] !== after[file] || modesBefore[file] !== modesAfter[file]).sort();
    assert.ok(changed.length > 0, 'No version delta');
    let versionChanges = 0;
    for (const owner of [...npm.values(), ...cargo.values()]) if (owner.before !== owner.after) versionChanges++;
    assert.ok(versionChanges > 0, 'No advancing package version');
    for (const row of npm.values()) npmManifest(JSON.parse(before[row.file]), JSON.parse(after[row.file]), npm);
    for (const row of cargo.values()) cargoManifest(toml(before[row.file]), toml(after[row.file]), cargo);
    for (const file of changed) {
      assert.ok([undefined, '100644'].includes(modesBefore[file]) && [undefined, '100644'].includes(modesAfter[file]), 'Mode or special-file change');
      const old = before[file], current = after[file];
      if (file.endsWith('/package.json') && old && current) npmManifest(JSON.parse(old), JSON.parse(current), npm);
      else if (file.endsWith('Cargo.toml') && old && current) cargoManifest(toml(old), toml(current), cargo, file === 'scripts/release-fixtures/sdk-consumer/Cargo.toml');
      else if (file === 'pnpm-lock.yaml' && old && current) pnpmLock(yaml(old), yaml(current), npm);
      else if (file === 'Cargo.lock' && old && current) cargoLock(toml(old), toml(current), cargo);
      else if (/^\.changeset\/[a-z0-9-]+\.md$/.test(file) && old && current === undefined) {
        assert.equal(receipt.consumed?.[file], hash(old), 'Changeset consumption differs');
      } else if (file.endsWith('/license-components.json') && old && current) {
        const expected = JSON.parse(old), manifest = path.posix.dirname(file) + '/package.json';
        assert.ok(before[manifest] && after[manifest]);
        for (const component of expected.components ?? []) for (const input of component.sourceFiles ?? []) {
          if (input.file === 'package.json') {
            assert.equal(input.sha256, hash(before[manifest].replaceAll('\r\n', '\n')));
            input.sha256 = hash(after[manifest].replaceAll('\r\n', '\n'));
          }
        }
        equal(expected, JSON.parse(current));
      } else if (file.endsWith('/CHANGELOG.md') && current) {
        const owner = [...npm.values(), ...cargo.values()].find(row => path.posix.dirname(row.file) === path.posix.dirname(file));
        assert.ok(owner && owner.before !== owner.after);
        assert.ok(current.includes('## ' + owner.after + '\n') && (!old || current.endsWith(old) || current.includes(old.slice(old.indexOf('\n') + 1))), 'Changelog rewrites existing history');
      } else {
        // Generated metadata is accepted only with an exact producer digest;
        // executable/configuration paths can never enter this escape hatch.
        assert.match(file, /^docs\/(?:releases\/v2-(?:candidate|release-intent)|release-evidence\/(?:changesets-(?:version-plan|preparation)|cargo-version-proposal))\.json$/);
        assert.equal(receipt.generated?.[file], hash(current ?? ''), 'Unverified generated metadata');
      }
    }
    return { kind: 'version-preparation-verification', mode: 'verified-version-only', changed,
      headSHA: receipt.headSHA, baseSHA: receipt.baseSHA, acceptanceInherited: false,
      diffHash: hash(JSON.stringify(changed.map(file => [file, before[file] ?? null, after[file] ?? null]))) };
  } catch (error) {
    return { kind: 'version-preparation-verification', mode: 'full', reason: error.message, acceptanceInherited: false };
  }
}
