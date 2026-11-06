import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
const read = name => fs.readFileSync(new URL('../' + name, import.meta.url), 'utf8');
const workflows = fs.readdirSync(new URL('../.github/workflows/', import.meta.url)).filter(name => /\.ya?ml$/.test(name));

test('every workflow is free of performance collection, base comparison and automatic reports', () => {
  for (const file of workflows) {
    const raw = read('.github/workflows/' + file);
    assert.doesNotMatch(raw, /performance-report|performance-evidence|prepare-performance-base|aggregate-performance-reports|performance-pr-reporter|PERFORMANCE_SAMPLING_RESULT|Commerce Performance/, file);
    const workflow = YAML.parse(raw);
    for (const [name, job] of Object.entries(workflow.jobs)) {
      for (const dependency of typeof job.needs === 'string' ? [job.needs] : job.needs ?? [])
        assert.ok(Object.hasOwn(workflow.jobs, dependency), `${file}: ${name} -> ${dependency}`);
    }
  }
  assert.equal(fs.existsSync(new URL('../.github/workflows/performance-report-comment.yaml', import.meta.url)), false);
});
test('ordinary execution baseline keeps real coverage and immutable source evidence', () => {
  const jobs = YAML.parse(read('.github/workflows/ci.yaml')).jobs;
  const job = jobs['execution-baseline'];
  assert.equal(job.name, 'Record verified CI execution baseline');
  for (const platform of ['linux', 'macos', 'windows']) assert.ok(job.needs.includes('primary-' + platform));
  for (const command of ['ci-suite-attempts.py', 'ci-suite-execution.mjs --collect', 'ci-execution-ledger.py'])
    assert.ok(job.steps.some(step => step.run?.includes(command)), command);
  assert.ok(job.steps.some(step => step.with?.name === 'farm-ci-execution-baseline-${{ github.run_id }}-${{ github.run_attempt }}' && step.with['if-no-files-found'] === 'error'));
  for (const name of ['v2-cargo-cache-command', 'verify-published-v2-consumers', 'ci-native-capture', 'unified-impact-graph'])
    assert.ok(YAML.parse(read('.github/workflows/ci-platform-safety.yaml')).jobs['platform-contract-tests'].steps.some(step => step.run?.includes(name)), name);
});
test('Commerce retains actual business assertions as ordinary production Examples E2E', () => {
  const spec = read('examples/commerce-lab/e2e.spec.mjs');
  assert.match(spec, /COMMERCE_SCALE: 'business-smoke'/);
  assert.match(spec, /verifyBusinessBrowser/);
  assert.match(spec, /expectedCategories: 4/);
  assert.match(spec, /'build'/);
  assert.match(spec, /verifyBuiltFarm/);
  assert.doesNotMatch(spec, /supervisedCommand|business-2048|medianMs/);
  const assertions = read('examples/commerce-lab/benchmark/business-correctness.mjs');
  for (const feature of ['Validate in Worker', 'Generate TypeScript types', 'Import drafts', 'Publish campaign', 'MDX table', 'getComputedStyle', 'health.assertHealthy()']) assert.ok(assertions.includes(feature), feature);
});
test('release and recovery require functional evidence but no performance archives', () => {
  for (const name of ['incremental-release-functional.py', 'incremental-chain-runtime.py', 'v2-auto-release.mjs', 'prepare-v2-release-intent.mjs'])
    assert.doesNotMatch(read('scripts/' + name), /performanceRequired|verify_performance|Commerce Performance|performance-pr-summary/, name);
  const functional = read('scripts/incremental-release-functional.py');
  for (const value of ['Typescript Test', 'Examples Test', 'Docs /', 'Type Check', 'main_binding_proof', 'bound_artifact', 'exact_plan', 'tested functional tree']) assert.ok(functional.includes(value), value);
  const release = YAML.parse(read('.github/workflows/incremental-release.yaml'));
  assert.ok(read('scripts/release-evidence-recovery.py').includes(release.jobs['main-evidence'].name));
  assert.ok(read('scripts/performance-report.mjs').includes('for (let round = -1; round < 3; round++)'));
});

test('Windows path safety receives the exact setup-python executable independently of sampling', () => {
  const caller = YAML.parse(read('.github/workflows/ci.yaml')).jobs['platform-contract-windows'];
  assert.equal(caller.with.os, 'windows-latest');
  assert.equal(caller.uses, './.github/workflows/ci-platform-safety.yaml');
  const job = YAML.parse(read('.github/workflows/ci-platform-safety.yaml')).jobs['platform-contract-tests'];
  const setup = job.steps.find(step => step.id === 'contract-python');
  assert.equal(setup.uses, 'actions/setup-python@v5');
  const step = job.steps.find(step => step.run?.includes('verify-published-v2-consumers.test.mjs'));
  assert.equal(step.env.FARM_CONTRACT_PYTHON, '${{ steps.contract-python.outputs.python-path }}');
  assert.match(read('scripts/verify-published-v2-consumers.test.mjs'), /const python = process.env.FARM_CONTRACT_PYTHON \?\? process.env.FARM_PERFORMANCE_PYTHON/);
});
