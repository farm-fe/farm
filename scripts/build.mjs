import { execa } from 'execa';
import { createSpinner } from 'nanospinner';
import { resolve, join } from 'node:path';
import fs from 'node:fs';

const DEFAULT_PACKAGE_MANAGER = 'pnpm';
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, './packages/core');

// Build rust_plugin_react
const PKG_RUST_PLUGIN = resolve(CWD, './rust-plugins');

// Build js_plugin_path
const PKG_JS_PLUGIN = resolve(CWD, './js-plugins');

// build core command
export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:rs'], { cwd: PKG_CORE });

// build rust plugins
export const rustPlugins = () => dynamicPlugin(PKG_RUST_PLUGIN);

// build js plugins
export const jsPlugins = () => dynamicPlugin(PKG_JS_PLUGIN);

export const buildJsPlugins = () => Promise.all(jsPlugins());

export const buildRustPlugins = () => Promise.all(rustPlugins());

export const copyArtifacts = () =>
  dynamicPlugin(PKG_RUST_PLUGIN, 'copy-artifacts');

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
  await runTask('RustPlugins', buildRustPlugins);
  await runTask('JsPlugins', buildJsPlugins);
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

function dynamicPlugin(baseDir, command = 'build', packageManager = 'pnpm') {
  const pluginNameMap = fs
    .readdirSync(baseDir)
    .filter((file) => fs.statSync(join(baseDir, file)).isDirectory());
  const path = pluginNameMap.map((subDir) => resolve(baseDir, subDir));
  return path.map((item) => {
    return execa(packageManager, [command], { cwd: item });
  });
}
