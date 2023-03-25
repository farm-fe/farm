import { execa } from 'execa';

import { createSpinner } from 'nanospinner';

import { resolve } from 'path';

const DEFAULT_PACKAGE_MANAGER = 'pnpm';
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, './packages/core');

// Build rust-plugin-react
const PKG_PLUGIN_REACT = resolve(CWD, './rust-plugins/react');

// Build js-plugin-vue
const PKG_PLUGIN_VUE = resolve(CWD, './js-plugins/vue');

export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:rs'], { cwd: PKG_CORE });

const plugins = [
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], { cwd: PKG_PLUGIN_REACT }),
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], { cwd: PKG_PLUGIN_VUE }),
];

export const buildPlugins = () => Promise.all(plugins);

export const copyArtifacts = () => {
  execa(DEFAULT_PACKAGE_MANAGER, ['copy-artifacts'], { cwd: PKG_PLUGIN_REACT });
};

export async function runTask(taskName, task) {
  const s = createSpinner(`${taskName}`).start();
  try {
    await task();
    s.success({ text: `${taskName} completed!` });
  } catch (e) {
    s.error({ text: `${taskName} failed!` });
    console.error(e.toString());
  }
}

export async function runTaskQueue() {
  await runTask('Build core', buildCore);
  await runTask('Build plugins', buildPlugins);
  await runTask('Copy Artifacts', copyArtifacts);
}
