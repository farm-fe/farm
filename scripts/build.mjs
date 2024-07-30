import fs, { existsSync } from 'node:fs';
import os from 'node:os';
import { join, resolve } from 'node:path';
import { execa } from 'execa';
import { createSpinner } from 'nanospinner';

import { logger } from './logger.mjs';

export const DEFAULT_PACKAGE_MANAGER = 'pnpm';
const DEFAULT_HOMEBREW_PACKAGE_MANAGER = 'brew';
const DEFAULT_LINUX_PACKAGE_MANAGER = 'apt';
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, './packages/core');

// Build cli
const PKG_CLI = resolve(CWD, './packages/cli');

// Build plugin-tools
const PKG_PLUGIN_TOOLS = resolve(CWD, './packages/plugin-tools');

// Build plugin dts
const PKG_DTS = resolve(CWD, './js-plugins/dts');

// Build rust_plugin_react
const PKG_RUST_PLUGIN = resolve(CWD, './rust-plugins');

// Build js_plugin_path
export const JS_PLUGINs_DIR = resolve(CWD, './js-plugins');
export const EXAMPLES_DIR = resolve(CWD, './examples');

export const excludedJsPlugin = ['dts'];

export const buildExamples = async () => {
  const examples = fs.readdirSync('./examples');
  console.log('Building', examples.length, 'examples...');

  for (const example of examples) {
    const examplePath = join('./examples', example);
    if (!existsSync(join(examplePath, 'package.json'))) {
      continue;
    }
    console.log('Building', examplePath);

    if (fs.statSync(examplePath).isDirectory()) {
      await execa('npm', ['run', 'build'], {
        cwd: examplePath
      });
    }
  }
};

export async function runTaskQueue() {
  // The sass plug-in uses protobuf, so you need to determine whether the user installs it or not.
  await installProtoBuf();
  await runTask('Cli', buildCli);
  await runTask('Core', buildCore);
  await runTask('PluginTools', buildPluginTools);
  await runTask('RustPlugins', buildRustPlugins);
  await runTask('JsPlugins', buildJsPlugins);
  await runTask('Artifacts', copyArtifacts);
}

// install mac protobuf
export const installMacProtobuf = () =>
  execa(DEFAULT_HOMEBREW_PACKAGE_MANAGER, ['install', 'protobuf'], {
    cwd: CWD
  });

// install linux protobuf
export const installLinuxProtobuf = async () => {
  try {
    await execa('type', DEFAULT_LINUX_PACKAGE_MANAGER);
  } catch (_) {
    return Promise.reject(
      `not found "${DEFAULT_LINUX_PACKAGE_MANAGER}", if it's not your package manager, please install "protobuf" manually.`
    );
  }

  return execa(
    DEFAULT_LINUX_PACKAGE_MANAGER,
    ['install', '-y', 'protobuf-compiler'],
    {
      cwd: CWD
    }
  );
};

// build core command
export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:rs'], {
    cwd: PKG_CORE
  }).then(buildCoreCjs);

export const buildCoreCjs = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build:cjs'], {
    cwd: PKG_CORE
  });

// build cli command
export const buildCli = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], {
    cwd: PKG_CLI
  });

// build farm-plugin-tools
export const buildPluginTools = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], {
    cwd: PKG_PLUGIN_TOOLS
  });

// build dts command
export const buildDts = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ['build'], {
    cwd: PKG_DTS
  });

// build rust plugins
export const rustPlugins = () => batchBuildPlugins(PKG_RUST_PLUGIN);

// // build js plugins
// export const jsPlugins = () => batchBuildPlugins(PKG_JS_PLUGIN);

// build chain
export const buildJsPlugins = async () => {
  await execa(
    DEFAULT_PACKAGE_MANAGER,
    ['--filter', './js-plugins/**', 'build'],
    {
      cwd: CWD
    }
  );

  // // First, build Dts
  // await buildDts();

  // // Then, build other js plugins
  // await Promise.all(jsPlugins());
};

export const buildRustPlugins = () => Promise.all(rustPlugins());

export const copyArtifacts = () =>
  batchBuildPlugins(PKG_RUST_PLUGIN, 'copy-artifacts');

export async function runTask(
  taskName,
  task,
  processText = 'Building',
  finishedText = 'Build'
) {
  const spinner = createSpinner(`${processText} ${taskName}`).start();
  try {
    await task();
    spinner.success({ text: `${finishedText} ${taskName} completed!` });
  } catch (e) {
    spinner.error({ text: `${finishedText} ${taskName} failed!` });
    console.error(e.toString());
    process.exit(1);
  }
}

export function resolveNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;

  if (requiredMajorVersion < minimumMajorVersion) {
    logger(`Farm does not support using Node.js v${currentVersion}!`);
    logger(`Please use Node.js v${minimumMajorVersion} or higher.`);
    process.exit(1);
  }
}

export function batchBuildPlugins(
  baseDir,
  command = 'build',
  packageManager = 'pnpm'
) {
  const pluginNameMap = fs.readdirSync(baseDir).filter((file) => {
    return (
      fs.statSync(join(baseDir, file)).isDirectory() &&
      !excludedJsPlugin.includes(file)
    );
  });
  const path = pluginNameMap.map((subDir) => resolve(baseDir, subDir));
  return path.map((item) => {
    return execa(packageManager, [command], { cwd: item });
  });
}

export function isMac() {
  const platform = os.platform();
  return platform === 'darwin';
}

export function isLinux() {
  const platform = os.platform();
  return platform === 'linux';
}

export function isWindows() {
  const platform = os.platform();
  return platform === 'win32';
}

export async function checkProtobuf() {
  const isWindowsFlag = isWindows();
  const isMacFlag = isMac();
  const isLinuxFlag = isLinux();
  try {
    if (isWindowsFlag) {
      await execa('where', ['protoc']);
    } else if (isMacFlag || isLinuxFlag) {
      await execa('which', ['protoc']);
    }
    return true;
  } catch {
    return false;
  }
}

export async function installProtoBuf() {
  const installFlag = await checkProtobuf();
  if (!installFlag) {
    logger(
      'Due to the use of protoc in the project, we currently judge that you have not installed. we need to install protobuf locally to make the project start successfully. \n\n- For mac users, will be use your local `homebrew` tool for installation. (First, Make sure your computer has `homebrew` installed) \n- For linux users, we will use your local `apt` tool for installation. (First, Make sure your computer has `apt` installed) \n- For Windows users, because the protobuf plugin cannot be installed automatically, You need to install manually according to the prompts \n',
      { title: 'FARM WARN', color: 'yellow' }
    );

    if (isMac()) {
      await runTask('Protobuf', installMacProtobuf, 'Install', 'Install');
    } else if (isLinux()) {
      await runTask('Protobuf', installLinuxProtobuf, 'Install', 'Install');
    }

    if (isWindows()) {
      logger(
        'If you are using a windows system, you can install it in the following ways:\n\n 1. open https://github.com/protocolbuffers/protobuf \n If you are a 32-bit operating system install https://github.com/protocolbuffers/protobuf/releases/download/v21.7/protoc-21.7-win32.zip \n If you are a 64-bit operating system install https://github.com/protocolbuffers/protobuf/releases/download/v21.7/protoc-21.7-win64.zip \n 2. After installation, find the path you installed, and copy the current path, adding to the environment variable of windows \n\n Or you can directly check out the following article to install \n https://www.geeksforgeeks.org/how-to-install-protocol-buffers-on-windows/',
        { title: 'FARM TIPS', color: 'yellow' }
      );
      process.exit(1);
    }
  } else {
    console.log('');
    logger('Protobuf has been installed, skipping installation. \n');
  }
}

export async function cleanBundleCommand() {
  try {
    await execa(DEFAULT_PACKAGE_MANAGER, [
      '-r',
      '--filter=./packages/*',
      '--filter=./js-plugins/*',
      'run',
      'clean'
    ]);
    logger('pnpm clean command completed successfully.');
  } catch (error) {
    logger('An error occurred while running pnpm clean command:', {
      title: error.message,
      color: 'red'
    });
    process.exit(1);
  }
}
