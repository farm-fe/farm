import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { buildStages, runStages, parseToolchain } from './farm-docs-vercel-build.mjs';

test('clean deployment builds native and public workspace prerequisites before the website', () => {
  const ids = buildStages.map(stage => stage.id);
  assert.equal(new Set(ids).size, ids.length);
  assert.ok(ids.indexOf('plugin-tools') < ids.indexOf('native:replace-dirname'));
  assert.ok(ids.indexOf('core:native') < ids.indexOf('core:public'));
  for (const name of ['replace-dirname', 'react', 'mdx', 'ssr-css', 'syntax-highlight', 'sass', 'tailwindcss', 'docs', 'docs-snapshot', 'docs-seo', 'docs-snippets']) {
    assert.ok(ids.indexOf(`native:${name}`) > ids.indexOf('core:native'));
    assert.ok(ids.indexOf(`native:${name}`) < ids.indexOf('core:public'));
  }
  assert.ok(ids.indexOf('cli:public') < ids.indexOf('website'));
  assert.ok(ids.indexOf('docs:public') < ids.indexOf('docs-theme-default:public'));
  assert.ok(ids.indexOf('docs-theme-default:public') < ids.indexOf('website'));
  for (const stage of buildStages.filter(stage => stage.script)) {
    const manifest = JSON.parse(fs.readFileSync(new URL(`../${stage.directory}/package.json`, import.meta.url)));
    assert.ok(manifest.scripts[stage.script], `${stage.id} must invoke an existing public script`);
  }
});

test('a failed prerequisite stops deployment before the website can consume partial artifacts', async () => {
  const visited = [];
  await assert.rejects(runStages(buildStages, async stage => {
    visited.push(stage.id);
    if (stage.id === 'core:native') throw new Error('native build failed');
  }), /native build failed/);
  assert.equal(visited.at(-1), 'core:native');
  assert.ok(!visited.includes('website'));
});

test('the website deployment explicitly installs the workspace and publishes the Farm output', () => {
  const config = JSON.parse(fs.readFileSync(new URL('../website/vercel.json', import.meta.url)));
  const manifest = JSON.parse(fs.readFileSync(new URL('../website/package.json', import.meta.url)));
  assert.equal(config.framework, null);
  assert.equal(config.outputDirectory, 'build-farm');
  assert.equal(config.installCommand, 'node ../scripts/farm-docs-vercel-build.mjs --install');
  assert.equal(config.buildCommand, 'node ../scripts/farm-docs-vercel-build.mjs');
  assert.equal(manifest.engines.node, '22.x');
});

test('the actual repository TOML supplies the pinned Rust channel and all required components', () => {
  const actual = parseToolchain(fs.readFileSync(new URL('../rust-toolchain.toml', import.meta.url), 'utf8'));
  assert.equal(actual.channel, 'nightly-2026-04-10');
  assert.deepEqual(actual.components, ['clippy', 'rustfmt', 'rust-src', 'rustc-dev']);
  assert.throws(() => parseToolchain('[toolchain]\nchannel = "nightly"\n'), /Cannot read/);
});


test('website relative source imports match exact tracked Git paths on every host', async () => {
  const { assertWebsiteSourcePaths, relativeImports, unresolvedImports } = await import('./farm-docs-source-paths.mjs');
  const tracked = new Set(['website/src/components/card/index.tsx', 'website/src/icon.svg']);
  const filename = 'website/src/components/Team/index.tsx';
  assert.equal(unresolvedImports(filename, ['../Card'], tracked).length, 1);
  assert.equal(unresolvedImports(filename, ['../Missing'], tracked).length, 1);
  assert.deepEqual(unresolvedImports(filename, ['../card', '../card/index.js', '../../icon.svg?url'], tracked), []);
  assert.deepEqual(relativeImports('import A from "../card"; export { A } from "../card"; import("./lazy"); // import("./fake")', filename), ['../card', '../card', './lazy']);
  assert.ok(assertWebsiteSourcePaths().checked > 0);
});


test('the deployment bootstrap starts before workspace dependencies are installed', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-bootstrap-'));
  try {
    const entry = path.join(directory, 'build.mjs');
    fs.copyFileSync(new URL('./farm-docs-vercel-build.mjs', import.meta.url), entry);
    const result = spawnSync(process.execPath, [entry, '--plan'], { encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(JSON.parse(result.stdout).length, buildStages.length);
  } finally {
    assert.ok(path.resolve(directory).startsWith(path.resolve(os.tmpdir()) + path.sep));
    fs.rmSync(directory, { recursive: true });
  }
});
