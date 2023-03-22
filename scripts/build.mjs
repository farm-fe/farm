import { execa } from 'execa';

import { createSpinner } from 'nanospinner';

import { resolve } from 'path';

const DEFAULT_PACKAGE_MANAGER = 'pnpm';
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, './packages/core');

// Build plugin-react
const PKG_PLUGIN_REACT = resolve(CWD, './rust-plugins/react');

export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:rs'], { cwd: PKG_CORE });

export const buildPluginReact = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], { cwd: PKG_PLUGIN_REACT });

export const copyArtifacts = () => {
  execa(DEFAULT_PACKAGE_MANAGER, ['copy-artifacts'], { cwd: PKG_PLUGIN_REACT });
};

export async function runTask(taskName, task) {
  const s = createSpinner(`${taskName}`).start();
  try {
    await task();
    s.success({ text: `${taskName} completed!` });
  } catch (e) {
    s.error({ text: `Build ${taskName} failed!` });
    console.error(e.toString());
  }
}

export async function runTaskQueue() {
  await runTask('Building core', buildCore);
  await runTask('Building plugins', buildPluginReact);
  await runTask('Copy Artifacts', copyArtifacts);
}
