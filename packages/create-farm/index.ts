#!/usr/bin/env node
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';
import spawn from 'cross-spawn';

import { fileURLToPath } from 'node:url';
import { colors } from './utils/color.js';

import { loadWithRocketGradient } from './utils/gradient.js';
import createSpawnCmd from './utils/createSpawnCmd.js';
import { shouldUseYarn, shouldUsePnpm } from './utils/packageManager.js';

interface IResultType {
  packageName?: string;
  projectName?: string;
  framework?: string;
  argFrameWork?: string;
  autoInstall?: boolean;
  packageManager?: string;
}

export const tauriTemplate = [
  {
    type: 'select',
    name: 'tauri-framework',
    message: 'Select a tauri framework:',
    initial: 0,
    choices: [
      {
        title: colors.cyan('React'),
        value: 'react'
      },
      { title: colors.green('Vue'), value: 'vue' },
      {
        title: colors.cyan('Preact'),
        value: 'preact'
      },
      { title: colors.blue('Solid'), value: 'solid' },
      { title: colors.orange('Svelte'), value: 'svelte' },
      {
        title: colors.yellow('Vanilla'),
        value: 'vanilla'
      }
    ]
  }
];

// judge node version
judgeNodeVersion();

// command
welcome();

// argv
const argv = minimist<{
  t?: string;
  template?: string;
  skipInstall: boolean;
  'skip-install': boolean;
}>(process.argv.slice(2), { string: ['_'] });

const cwd = process.cwd();
const isYarnInstalled = shouldUseYarn();
const isPnpmInstalled = shouldUsePnpm();
const DEFAULT_TARGET_NAME = 'farm-project';
const pkgInfo = pkgFromUserAgent(process.env.npm_config_user_agent);
async function createFarm() {
  const argProjectName = formatTargetDir(argv._[0]);
  const argFramework = argv.template || argv.t;
  let targetDir = argProjectName || DEFAULT_TARGET_NAME;
  let result: IResultType = {};
  const skipInstall = argv['skip-install'] ?? argv.skipInstall ?? true;
  try {
    result = await prompts(
      [
        {
          type: argProjectName ? null : 'text',
          name: 'projectName',
          message: 'Project name:',
          initial: DEFAULT_TARGET_NAME,
          onState: (state) => {
            targetDir = formatTargetDir(state.value) || DEFAULT_TARGET_NAME;
          }
        },
        {
          type: () =>
            !fs.existsSync(targetDir) || isEmpty(targetDir) ? null : 'confirm',
          name: 'overwrite',
          message: () =>
            (targetDir === '.'
              ? '🚨 Current directory'
              : `🚨 Target directory "${targetDir}"`) +
            ` is not empty. Overwrite existing files and continue?`
        },
        {
          type: (_, { overwrite }: { overwrite?: boolean }) => {
            if (overwrite === false) {
              throw new Error(colors.red('❌') + ' Operation cancelled');
            }
            return null;
          },
          name: 'overwriteChecker'
        },
        {
          type: argFramework ? null : 'select',
          name: 'framework',
          message: 'Select a framework:',
          initial: 0,
          choices: [
            {
              title: colors.cyan('React'),
              value: 'react'
            },
            { title: colors.green('Vue'), value: 'vue' },
            {
              title: colors.cyan('Preact'),
              value: 'preact'
            },
            { title: colors.blue('Solid'), value: 'solid' },
            { title: colors.orange('Svelte'), value: 'svelte' },
            {
              title: colors.yellow('Vanilla'),
              value: 'vanilla'
            },
            { title: colors.red('Lit'), value: 'lit' },
            { title: colors.orange('Tauri'), value: 'tauri' }
          ]
        },
        {
          type: pkgInfo || skipInstall ? null : 'select',
          name: 'packageManager',
          message: 'Which package manager do you want to use?',
          choices: [
            { title: 'npm', value: 'npm' },
            {
              title: isYarnInstalled ? 'Yarn' : 'Yarn (not installed)',
              value: 'yarn',
              disabled: !isYarnInstalled
            },
            {
              title: isPnpmInstalled ? 'Pnpm' : 'Pnpm (not installed)',
              value: 'pnpm',
              disabled: !isPnpmInstalled
            }
          ]
        }
      ],
      {
        onCancel: () => {
          throw new Error(colors.red('❌') + ' Operation cancelled');
        }
      }
    );
  } catch (cancelled) {
    console.log(cancelled.message);
    return;
  }
  const { framework = argFramework, packageManager } = result;
  let chooseFramework: IResultType['framework'] = framework;
  if (framework === 'tauri') {
    const tauriOption = await prompts(tauriTemplate as prompts.PromptObject[]);
    chooseFramework = `tauri/${tauriOption['tauri-framework']}`;
  }

  await copyTemplate(targetDir, {
    framework: chooseFramework,
    projectName: targetDir,
    packageManager
  });
  await installationDeps(targetDir, !skipInstall, result);
}

function formatTargetDir(targetDir: string | undefined) {
  return targetDir?.trim().replace(/\/+$/g, '');
}

function isEmpty(path: string) {
  const files = fs.readdirSync(path);
  return files.length === 0 || (files.length === 1 && files[0] === '.git');
}

async function copyTemplate(targetDir: string, options: IResultType) {
  const spinner = await loadWithRocketGradient('Copy template');
  const dest = path.join(cwd, targetDir);
  const templatePath = path.join(
    fileURLToPath(import.meta.url),
    `../../templates/${options.framework}`
  );
  copy(templatePath, dest);

  writePackageJson(dest, options);
  spinner.text = 'Template copied Successfully!';
  spinner.succeed();
}

function writePackageJson(dest: string, options: IResultType) {
  const pkg = JSON.parse(
    fs.readFileSync(path.join(dest, `package.json`), 'utf-8')
  );

  pkg.name = options.projectName;

  const currentPkgManager = getCurrentPkgManager(options);
  if (currentPkgManager === 'yarn') {
    pkg.scripts = pkg.scripts ?? {};
    pkg.scripts.postinstall = 'npx --yes peer-gear --install';
  }

  const packageJsonPath = path.join(dest, 'package.json');
  const { name, ...rest } = pkg;
  const sortedPackageJson = { name, ...rest };
  fs.writeFileSync(
    packageJsonPath,
    JSON.stringify(sortedPackageJson, null, 2) + '\n'
  );
}

function getCurrentPkgManager(options: IResultType) {
  const pkgManager = pkgInfo ? pkgInfo.name : 'npm';
  const currentPkgManager =
    (pkgInfo ? pkgManager : options.packageManager) ?? 'npm';
  return currentPkgManager;
}

async function installationDeps(
  targetDir: string,
  autoInstall: boolean,
  options: IResultType
) {
  const currentPkgManager = getCurrentPkgManager(options);
  if (autoInstall) {
    const cmdInherit = createSpawnCmd(path.resolve(cwd, targetDir), 'ignore');
    const spinner = await loadWithRocketGradient('Install Dependencies');
    await cmdInherit(
      currentPkgManager,
      currentPkgManager === 'pnpm'
        ? ['install', '--no-frozen-lockfile']
        : ['install']
    );
    spinner.text = 'Dependencies Installed Successfully!';
    spinner.succeed();
  }
  colors.handleBrandText(
    '\n > Initial Farm Project created successfully ✨ ✨ \n'
  );
  colors.handleBrandText(`   cd ${targetDir} \n`);

  autoInstall
    ? autoInstallText(currentPkgManager)
    : colors.handleBrandText(
        `   ${currentPkgManager} install \n\n   ${autoInstallText(
          currentPkgManager
        )}`
      );
}

function autoInstallText(currentPkgManager: string) {
  return `${currentPkgManager} ${
    currentPkgManager === 'npm' ? 'run start' : 'start'
  } `;
}

function pkgFromUserAgent(userAgent: string | undefined) {
  if (!userAgent) return undefined;
  const pkgSpec = userAgent.split(' ')[0];
  const pkgSpecArr = pkgSpec.split('/');
  return {
    name: pkgSpecArr[0],
    version: pkgSpecArr[1]
  };
}

function judgeNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;
  if (requiredMajorVersion < minimumMajorVersion) {
    console.log(
      colors.yellow(`create-farm unsupported Node.js v${currentVersion}.`)
    );
    console.log(
      colors.yellow(`Please use Node.js v${minimumMajorVersion} or higher.`)
    );
    process.exit(1);
  }
}

function copy(src: string, dest: string) {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    copyDir(src, dest);
  } else {
    fs.copyFileSync(src, dest);
  }
}

function copyDir(srcDir: string, destDir: string) {
  fs.mkdirSync(destDir, { recursive: true });
  for (const file of fs.readdirSync(srcDir)) {
    const srcFile = path.resolve(srcDir, file);
    const destFile = path.resolve(destDir, file);
    if (file === 'gitignore') {
      copy(srcFile, destFile);
      fs.renameSync(destFile, path.resolve(destDir, '.gitignore'));
    } else {
      copy(srcFile, destFile);
    }
  }
}

function welcome() {
  console.log(colors.BrandText('⚡ Welcome To Farm ! '));
}

createFarm();
