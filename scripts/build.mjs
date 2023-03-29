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
  const spinner = createSpinner(`Building ${taskName}`).start();
  try {
    await task();
    spinner.success({ text: `Build ${taskName} completed!` });
  } catch (e) {
    spinner.error({ text: `Build ${taskName} failed!` });
    console.error(e.toString());
  }
}

export async function runTaskQueue() {
  await runTask('Core', buildCore);
  await runTask('Plugins', buildPlugins);
  await runTask('Artifacts', copyArtifacts);
}

export function resolveNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;

  if (requiredMajorVersion < minimumMajorVersion) {
    console.error(`Farm does not support using Node.js v${currentVersion}!`);
    console.error(`Please use Node.js v${minimumMajorVersion} or higher.`);
    process.exit(1);
  }
}
