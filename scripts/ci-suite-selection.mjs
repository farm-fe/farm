import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { readGitSuiteFiles, discoverSuiteInventory, suitePolicy, makeSuiteExecutionPlan, suiteCaseExclusions } from './ci-suite-inventory.mjs';
import { suiteImpact } from './ci-suite-impact.mjs';

const digest = value => crypto.createHash('sha256').update(typeof value === 'string' || Buffer.isBuffer(value) ? value : JSON.stringify(value)).digest('hex');
const root = fileURLToPath(new URL('../', import.meta.url));
const git = (directory, args, options = {}) => execFileSync('git', args, { cwd: directory, timeout: 60000, maxBuffer: 32 * 1024 ** 2, ...options });

export function calculateSelection(identity, directory = root, read = (args, options) => git(directory, args, options)) {
  assert.equal(identity.kind, 'independently-verified-ci-suite-baseline');
  for (const key of ['sourceSHA', 'sourceTree', 'baselineSHA', 'baselineTree']) assert.match(identity[key] ?? '', /^[a-f0-9]{40}$/);
  const current = readGitSuiteFiles(directory, read);
  assert.equal(current.sourceIdentity.commit, identity.sourceSHA); assert.equal(current.sourceIdentity.tree, identity.sourceTree);
  assert.equal(read(['rev-parse', identity.baselineSHA + '^{tree}'], { encoding: 'utf8' }).trim(), identity.baselineTree);
  const previous = readGitSuiteFiles(directory, (args, options) => read(args.map(value => value === 'HEAD' ? identity.baselineSHA : value === 'HEAD^{tree}' ? identity.baselineSHA + '^{tree}' : value), options));
  const inventory = discoverSuiteInventory(current.files, suitePolicy, current.sourceIdentity, current.exampleDirectories);
  const names = new TextDecoder('utf-8', { fatal: true }).decode(read(['diff', '--name-only', '-z', '--no-renames', identity.baselineSHA, identity.sourceSHA])).split('\0').filter(Boolean);
  const decision = suiteImpact({ before: previous.files, after: current.files, changes: names.map(file => ({ path: file })), inventory });
  return { schemaVersion: 1, kind: 'ci-suite-execution-selection', ...identity, inventoryHash: inventory.inventoryHash,
    decision, decisionHash: decision.decisionHash, acceptanceInherited: false };
}

export function executionPlan(inventory, kind, directory = root, env = process.env) {
  if (!env.FARM_SUITE_SELECTION_HASH) return makeSuiteExecutionPlan(inventory, kind);
  assert.match(env.FARM_SUITE_SELECTION_HASH, /^[a-f0-9]{64}$/);
  const file = path.resolve(directory, 'ci-suite-selection/selection.json');
  assert.ok(fs.lstatSync(file).isFile() && !fs.lstatSync(file).isSymbolicLink() && fs.statSync(file).size <= 2 * 1024 ** 2);
  const bytes = fs.readFileSync(file); assert.equal(digest(bytes), env.FARM_SUITE_SELECTION_HASH, 'Selection does not match trusted job output');
  const value = JSON.parse(bytes);
  assert.equal(value.sourceSHA, env.GITHUB_SHA); assert.equal(value.runId, Number(env.GITHUB_RUN_ID));
  assert.equal(value.attempt, Number(env.CI_SUITE_SELECTION_PRODUCER_ATTEMPT ?? env.GITHUB_RUN_ATTEMPT)); assert.equal(value.repositoryId, Number(env.GITHUB_REPOSITORY_ID));
  assert.ok(Number.isSafeInteger(value.attempt) && value.attempt > 0 && value.attempt <= Number(env.GITHUB_RUN_ATTEMPT), 'Future selection attempt');
  const actual = calculateSelection(value, directory);
  assert.deepEqual(actual, value, 'Actual suite impact differs from bound selection');
  assert.equal(value.inventoryHash, inventory.inventoryHash);
  const selected = value.decision.sets[kind]; assert.ok(selected, 'Unknown suite kind');
  const result = { schemaVersion: 1, protocol: inventory.protocol, inventoryHash: inventory.inventoryHash, kind,
    mode: selected.mode, ids: selected.ids, selectionHash: env.FARM_SUITE_SELECTION_HASH, executionEnabled: true };
  if (kind === 'examples-browser') result.excludedCases = suiteCaseExclusions(inventory, kind, result.ids);
  return { ...result, planHash: digest(result) };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const input = JSON.parse(fs.readFileSync(0, 'utf8'));
  process.stdout.write(JSON.stringify(calculateSelection(input)) + '\n');
}
