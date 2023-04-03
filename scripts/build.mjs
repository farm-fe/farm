import { execa } from 'execa';
import { createSpinner } from 'nanospinner';
import { resolve, join } from 'node:path';
import fs from 'node:fs';

const DEFAULT_PACKAGE_MANAGER = 'pnpm';
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, './packages/core');

// Build rust-plugin-react
const PKG_RUST_PLUGIN = resolve(CWD, './rust-plugins');

// Build js-plugin-vue
const PKG_JS_PLUGIN = resolve(CWD, './js-plugins');

const jsPathMap = fs
  .readdirSync(PKG_JS_PLUGIN)
  .filter((file) => fs.statSync(join(PKG_JS_PLUGIN, file)).isDirectory());

const rustPathMap = fs
  .readdirSync(PKG_RUST_PLUGIN)
  .filter((file) => fs.statSync(join(PKG_RUST_PLUGIN, file)).isDirectory());
const rustPlugin = rustPathMap.map((subDir) =>
  resolve(PKG_RUST_PLUGIN, subDir)
);

export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:rs'], { cwd: PKG_CORE });

const rustPlugins = () => {
  return rustPlugin.map((item) => {
    return execa(DEFAULT_PACKAGE_MANAGER, ['build'], { cwd: item });
  });
};

const jsPlugin = jsPathMap.map((subDir) => resolve(PKG_JS_PLUGIN, subDir));

const jsPlugins = () => {
  return jsPlugin.map((item) => {
    return execa(DEFAULT_PACKAGE_MANAGER, ['build'], { cwd: item });
  });
};

export const buildJsPlugins = () => Promise.all(jsPlugins());

export const buildRustPlugins = () => Promise.all(rustPlugins());

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
  // await runTask('Core', buildCore);
  await runTask('rustPlugins', buildRustPlugins);
  await runTask('jsPlugins', buildJsPlugins);
  // await runTask('Artifacts', copyArtifacts);
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

function dynamicPlugin(baseDir)
