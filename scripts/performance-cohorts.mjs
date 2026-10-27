import fs from 'node:fs/promises';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { inflateRawSync } from 'node:zlib';
import { createHash } from 'node:crypto';

export const nativePlugins = ['sass', 'react', 'replace-dirname', 'vue', 'vue-jsx', 'auto-import', 'compress', 'dsv', 'dts', 'icons', 'image', 'mdx', 'modular-import', 'react-components', 'strip', 'svgr', 'url', 'virtual', 'wasm', 'worker', 'yaml', 'tailwindcss'];
const sha = value => createHash('sha256').update(value).digest('hex');
const validSha = value => /^[a-f0-9]{40}$/.test(value ?? '');
const unavailable = reason => ({ mode: 'current-only', reason, root: null, revision: null, nativeProvenance: null });
export function githubClient({ token = process.env.GITHUB_TOKEN, fetchImpl = fetch } = {}) {
  return async (route, binary = false) => {
    if (!token) throw new Error('GITHUB_TOKEN is unavailable');
    const remaining = Number(process.env.FARM_PERFORMANCE_STARTED_AT ?? Date.now()) + 18 * 60 * 1000 - Date.now();
    if (!Number.isFinite(remaining) || remaining <= 0) throw new Error('Performance metadata request budget exhausted');
    const response = await fetchImpl(`https://api.github.com${route}`, { headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' }, signal: AbortSignal.timeout(Math.min(30000, remaining)) });
    if (!response.ok) throw new Error(`GitHub REST ${response.status}`);
    if (!binary) return response.json();
    if (Number(response.headers.get('content-length')) > 128 * 1024 * 1024) throw new Error('Artifact exceeds download bound');
    const bytes = Buffer.from(await response.arrayBuffer());
    if (bytes.length > 128 * 1024 * 1024) throw new Error('Artifact exceeds download bound');
    return bytes;
  };
}
function repositoryRoute(repository) {
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository ?? '')) throw new Error('Invalid repository');
  return `/repos/${repository}`;
}
function trusted(run, repository, workflow) {
  return run.repository?.full_name === repository && run.head_repository?.full_name === repository && ['push', 'pull_request'].includes(run.event) && run.conclusion === 'success' && run.path === `.github/workflows/${workflow}` && validSha(run.head_sha);
}
export async function runArtifacts(api, prefix, runId) {
  const rows = [];
  for (let page = 1; page <= 10; page++) {
    const result = await api(`${prefix}/actions/runs/${runId}/artifacts?per_page=100${page === 1 ? '' : `&page=${page}`}`);
    rows.push(...(result.artifacts ?? []));
    if (rows.length >= (result.total_count ?? rows.length) || (result.artifacts ?? []).length < 100) return rows;
  }
  throw new Error('Artifact count exceeds bounded pagination');
}
export async function discoverBaseCohort({ repository, revision, abi, workflow = 'ci.yaml', api = githubClient() }) {
  const prefix = repositoryRoute(repository);
  if (!validSha(revision) || !/^[\w-]+$/.test(abi ?? '')) return unavailable('Invalid exact base revision or ABI');
  try {
    const runs = (await api(`${prefix}/actions/workflows/${encodeURIComponent(workflow)}/runs?head_sha=${revision}&status=success&per_page=10`)).workflow_runs ?? [];
    const approvedHeads = new Set([revision]);
    if (!runs.length) {
      // CI is PR-only: a main merge commit often has no run under its own SHA.
      // Resolve its merged PR through GitHub instead of waiting for nonexistent main CI.
      const pulls = await api(`${prefix}/commits/${revision}/pulls?per_page=10`);
      for (const pr of (Array.isArray(pulls) ? pulls : []).filter(pr => pr.merged_at && pr.merge_commit_sha === revision && pr.head?.repo?.full_name === repository).slice(0, 3)) {
        approvedHeads.add(pr.head.sha);
        runs.push(...((await api(`${prefix}/actions/workflows/${encodeURIComponent(workflow)}/runs?head_sha=${pr.head.sha}&status=success&per_page=10`)).workflow_runs ?? []));
      }
    }
    for (const run of runs.filter(run => trusted(run, repository, workflow) && approvedHeads.has(run.head_sha)).slice(0, 3)) {
      const artifacts = await runArtifacts(api, prefix, run.id);
      let producerSHA = revision;
      const core = artifacts.find(item => new RegExp(`^[a-f0-9]{40}-${abi}$`).test(item.name) && !item.expired);
      if (core && !core.name.startsWith(`${revision}-`)) {
        const candidate = core.name.slice(0, 40);
        const [source, producer] = await Promise.all([api(`${prefix}/git/commits/${revision}`), api(`${prefix}/git/commits/${candidate}`)]);
        if (!source.tree?.sha || source.tree.sha !== producer.tree?.sha) continue;
        producerSHA = candidate;
      }
      const required = ['', ...nativePlugins.map(name => `-plugin-${name}`)].map(suffix => `${producerSHA}-${abi}${suffix}`);
      const selected = required.map(name => artifacts.find(artifact => artifact.name === name && !artifact.expired));
      if (selected.every(Boolean)) return { mode: 'paired', revision, producerSHA, repository, abi, runId: run.id, artifacts: selected.map(({ id, name }) => ({ id, name })), reason: 'Exact base source tree and ABI artifacts from a successful trusted workflow run' };
    }
    return unavailable('Missing or expired complete exact-base-SHA native artifact cohort');
  } catch (error) { return unavailable(`Base artifact discovery unavailable: ${error.message}`); }
}
// Central-directory validation prevents traversal, symlinks, overlapping local names and ZIP bombs.
export function readSafeZip(bytes) {
  if (!Buffer.isBuffer(bytes) || bytes.length > 128 * 1024 * 1024) throw new Error('Invalid ZIP size');
  let end = -1;
  for (let i = bytes.length - 22; i >= Math.max(0, bytes.length - 65557); i--) if (bytes.readUInt32LE(i) === 0x06054b50) { end = i; break; }
  if (end < 0 || bytes.readUInt16LE(end + 4) || bytes.readUInt16LE(end + 6)) throw new Error('Unsupported ZIP');
  const count = bytes.readUInt16LE(end + 10);
  let cursor = bytes.readUInt32LE(end + 16), total = 0;
  const files = new Map();
  if (count > 500 || cursor + bytes.readUInt32LE(end + 12) > end) throw new Error('ZIP directory exceeds bounds');
  for (let i = 0; i < count; i++) {
    if (cursor + 46 > end || bytes.readUInt32LE(cursor) !== 0x02014b50) throw new Error('Invalid ZIP directory');
    const flags = bytes.readUInt16LE(cursor + 8), method = bytes.readUInt16LE(cursor + 10), compressed = bytes.readUInt32LE(cursor + 20), size = bytes.readUInt32LE(cursor + 24);
    const length = bytes.readUInt16LE(cursor + 28), extra = bytes.readUInt16LE(cursor + 30), comment = bytes.readUInt16LE(cursor + 32), offset = bytes.readUInt32LE(cursor + 42);
    const name = bytes.subarray(cursor + 46, cursor + 46 + length).toString('utf8');
    const mode = bytes.readUInt32LE(cursor + 38) >>> 16;
    if (!name || name.includes('\\') || name.includes('\0') || name.includes(':') || name.startsWith('/') || name.split('/').some(part => part === '..' || part === '.') || (mode & 0xf000) === 0xa000 || flags & 1 || ![0, 8].includes(method) || files.has(name)) throw new Error('Unsafe ZIP entry');
    total += size;
    if (total > 256 * 1024 * 1024 || offset + 30 > cursor || bytes.readUInt32LE(offset) !== 0x04034b50) throw new Error('ZIP entry exceeds bounds');
    const localLength = bytes.readUInt16LE(offset + 26), localExtra = bytes.readUInt16LE(offset + 28);
    if (bytes.subarray(offset + 30, offset + 30 + localLength).toString('utf8') !== name) throw new Error('ZIP local name mismatch');
    const start = offset + 30 + localLength + localExtra;
    if (start + compressed > cursor) throw new Error('ZIP payload exceeds bounds');
    if (!name.endsWith('/')) {
      const payload = bytes.subarray(start, start + compressed);
      const content = method === 0 ? payload : inflateRawSync(payload, { maxOutputLength: Math.max(1, size) });
      if (content.length !== size) throw new Error('ZIP size mismatch');
      files.set(name, content);
    }
    cursor += 46 + length + extra + comment;
  }
  return files;
}
export async function prepareBaseConsumer({ baseRoot, revision, cohort, protocol, api = githubClient() }) {
  try {
    if (cohort.mode !== 'paired' || cohort.revision !== revision) return unavailable(cohort.reason ?? 'No exact native cohort');
    if (execFileSync('git', ['rev-parse', 'HEAD'], { cwd: baseRoot, encoding: 'utf8' }).trim() !== revision) return unavailable('Base checkout SHA mismatch');
    const source = await fs.readFile(path.join(baseRoot, 'scripts/performance-report.mjs'), 'utf8');
    if (!source.includes(`'${protocol}'`)) return unavailable('Base source lacks compatible sampling protocol');
    for (const name of ['examples/commerce-lab/farm.rust.config.ts', 'examples/commerce-lab/benchmark/extension-generator.mjs', 'examples/commerce-lab/benchmark/native-provenance.mjs']) await fs.access(path.join(baseRoot, name));
    const nativeProvenance = [];
    for (const artifact of cohort.artifacts) {
      const plugin = artifact.name.slice(`${cohort.producerSHA ?? revision}-${cohort.abi}`.length).replace(/^-plugin-/, '');
      if (plugin && !nativePlugins.includes(plugin)) throw new Error('Unexpected native artifact');
      const files = readSafeZip(await api(`${repositoryRoute(cohort.repository)}/actions/artifacts/${artifact.id}/zip`, true));
      const expected = plugin ? 'index.farm' : `farm.${cohort.abi}.node`;
      if (files.size !== 1 || !files.has(expected)) throw new Error(`Unexpected artifact contents: ${artifact.name}`);
      const destination = plugin ? path.join(baseRoot, 'rust-plugins', plugin, 'npm', cohort.abi) : path.join(baseRoot, 'packages/core/binding');
      await fs.mkdir(destination, { recursive: true });
      await fs.writeFile(path.join(destination, expected), files.get(expected));
      nativeProvenance.push({ artifactId: artifact.id, name: artifact.name, sha256: sha(files.get(expected)) });
    }
    return { mode: 'paired', reason: cohort.reason, root: path.resolve(baseRoot), revision, nativeProvenance, runId: cohort.runId };
  } catch (error) { return unavailable(`Base consumer unavailable: ${error.message}`); }
}
export const historyCompatibilityKeys = ['protocol', 'fixtureSha256', 'node', 'pnpm', 'os', 'arch', 'runnerImage', 'runnerImageVersion', 'lockSha256'];
export async function loadComparableHistory({ repository, abi, metadata, workflow = 'ci.yaml', api = githubClient(), limit = 10, currentRunId = process.env.GITHUB_RUN_ID }) {
  const result = { entries: [], excluded: [], status: 'missing' };
  try {
    const prefix = repositoryRoute(repository);
    const runs = (await api(`${prefix}/actions/workflows/${encodeURIComponent(workflow)}/runs?status=success&per_page=20`)).workflow_runs ?? [];
    for (const run of runs.filter(run => trusted(run, repository, workflow) && String(run.id) !== String(currentRunId)).slice(0, Math.min(limit, 10))) {
      const artifact = (await runArtifacts(api, prefix, run.id)).find(item => new RegExp(`^performance-[a-f0-9]{40}-${abi}${(run.run_attempt ?? 1) === 1 ? '(?:-attempt-1)?' : `-attempt-${run.run_attempt}`}$`).test(item.name));
      if (!artifact || artifact.expired) { result.excluded.push({ runId: run.id, reason: artifact ? 'expired' : 'missing' }); continue; }
      try {
        const files = readSafeZip(await api(`${prefix}/actions/artifacts/${artifact.id}/zip`, true));
        const raw = files.get('summary.json');
        if (!raw || raw.length > 2 * 1024 * 1024) throw new Error('Missing or oversized summary');
        const report = JSON.parse(raw.toString('utf8'));
        const producerSHA = artifact.name.slice('performance-'.length, 'performance-'.length + 40);
        if (producerSHA !== run.head_sha) {
          const [head, producer] = await Promise.all([api(`${prefix}/git/commits/${run.head_sha}`), api(`${prefix}/git/commits/${producerSHA}`)]);
          if (!head.tree?.sha || head.tree.sha !== producer.tree?.sha) throw new Error('Historical producer tree differs from API-confirmed source');
        }
        if (report.status !== 'complete' || report.metadata?.revision !== producerSHA || historyCompatibilityKeys.some(key => metadata[key] === undefined || report.metadata[key] !== metadata[key])) { result.excluded.push({ runId: run.id, reason: 'incompatible' }); continue; }
        const cells = (report.cells ?? []).filter(cell => ['cold', 'warm'].includes(cell.cacheMode) && Number.isFinite(cell.medianMs) && Number.isFinite(cell.iqrMs) && cell.validSamples >= 3).map(({ cacheMode, medianMs, iqrMs, validSamples }) => ({ cacheMode, medianMs, iqrMs, validSamples }));
        if (cells.length !== 2) throw new Error('Invalid sample summary');
        result.entries.push({ revision: producerSHA, sourceHeadSHA: run.head_sha, event: run.event, runId: run.id, artifactId: artifact.id, createdAt: run.created_at, os: metadata.os, arch: metadata.arch, cells });
      } catch (error) { result.excluded.push({ runId: run.id, reason: 'invalid', detail: error.message }); }
    }
    result.status = result.entries.length ? 'available' : result.excluded.some(row => row.reason === 'incompatible') ? 'incompatible' : 'missing';
  } catch (error) { result.status = 'unavailable'; result.error = error.message; }
  return result;
}

