import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile
} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { expect, test } from 'vitest';
import { mergeFarmCliConfig } from '../src/config/merge-config.js';
import { parseUserConfig } from '../src/config/schema.js';
import type { ResolvedUserConfig } from '../src/config/types.js';
import { build, preview, start, watch } from '../src/index.js';
import { runConfiguredWorkflow } from '../src/workflow.js';

test('package workflows resolve from a relative project root', async () => {
  const root = await mkdtemp(
    path.join(await realpath(os.tmpdir()), 'farm-workflow-relative-')
  );
  try {
    const runner = path.join(root, 'node_modules', 'fixture-workflow');
    await mkdir(runner, { recursive: true });
    await writeFile(
      path.join(runner, 'package.json'),
      JSON.stringify({
        name: 'fixture-workflow',
        type: 'module',
        exports: './index.mjs'
      })
    );
    await writeFile(
      path.join(runner, 'index.mjs'),
      'export async function run({config}) { if (!config.root) throw new Error("missing root"); }'
    );
    const relativeRoot = path.relative(process.cwd(), root);
    expect(path.isAbsolute(relativeRoot)).toBe(false);
    expect(
      await runConfiguredWorkflow(
        {
          root: relativeRoot,
          workflow: { module: 'fixture-workflow' }
        } as ResolvedUserConfig,
        'build'
      )
    ).toBe(true);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('workflow configuration is validated and preserved for inline callers', () => {
  const workflow = {
    module: './docs-workflow.mjs',
    options: { title: 'Docs' }
  };
  expect(parseUserConfig({ workflow }).workflow).toEqual(workflow);
  expect(mergeFarmCliConfig({ workflow }, {}, 'production').workflow).toEqual(
    workflow
  );
  expect(() => parseUserConfig({ workflow: { module: '' } })).toThrow();
  expect(() =>
    parseUserConfig({
      workflow: { module: './runner.mjs', unexpected: true }
    } as never)
  ).toThrow();
});

test('ordinary projects do not load or execute a workflow', async () => {
  expect(
    await runConfiguredWorkflow(
      { root: process.cwd() } as ResolvedUserConfig,
      'build'
    )
  ).toBe(false);
});

test('workflow receives the command, options and a config without recursive dispatch', async () => {
  const root = await mkdtemp(
    path.join(await realpath(os.tmpdir()), 'farm-workflow-')
  );
  try {
    await writeFile(
      path.join(root, 'runner.mjs'),
      `
      import { writeFile } from 'node:fs/promises';
      import path from 'node:path';
      export async function run({command, options, config}) {
        await writeFile(path.join(config.root, command + '.json'), JSON.stringify({command, options, recursive: !!config.workflow}));
      }
    `
    );
    const workflow = { module: './runner.mjs', options: { title: 'Docs' } };
    const config = { root, workflow } as ResolvedUserConfig;
    for (const command of ['build', 'dev', 'preview', 'watch'] as const) {
      expect(await runConfiguredWorkflow(config, command)).toBe(true);
      expect(
        JSON.parse(await readFile(path.join(root, `${command}.json`), 'utf8'))
      ).toEqual({ command, options: { title: 'Docs' }, recursive: false });
      await rm(path.join(root, `${command}.json`));
      const commands = { build, dev: start, preview, watch };
      await commands[command]({
        root,
        configFile: false,
        compilation: {
          input: { index: './intentionally-not-compiled.js' },
          persistentCache: false
        },
        workflow
      });
      expect(
        JSON.parse(await readFile(path.join(root, `${command}.json`), 'utf8'))
          .command
      ).toBe(command);
    }
    expect(config.workflow).toBe(workflow);
    await writeFile(
      path.join(root, 'broken.mjs'),
      'export async function run() { throw new Error("workflow failure"); }'
    );
    await expect(
      runConfiguredWorkflow(
        {
          root,
          workflow: {
            module: pathToFileURL(path.join(root, 'broken.mjs')).href
          }
        } as ResolvedUserConfig,
        'build'
      )
    ).rejects.toThrow('workflow failure');
    for (const action of [build, start, preview, watch]) {
      await expect(
        action({
          root,
          configFile: false,
          compilation: {
            input: { index: './intentionally-not-compiled.js' },
            persistentCache: false
          },
          workflow: { module: './broken.mjs' }
        })
      ).rejects.toThrow('workflow failure');
    }
    await writeFile(
      path.join(root, 'missing-run.mjs'),
      'export const other = true;'
    );
    await expect(
      runConfiguredWorkflow(
        {
          root,
          workflow: { module: './missing-run.mjs' }
        } as ResolvedUserConfig,
        'build'
      )
    ).rejects.toThrow('FARM_WORKFLOW_EXPORT');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
