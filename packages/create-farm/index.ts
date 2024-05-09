#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import minimist from 'minimist';
import prompts from 'prompts';

import { fileURLToPath } from 'node:url';
import { colors } from './utils/color.js';

import createSpawnCmd from './utils/createSpawnCmd.js';
import { loadWithRocketGradient } from './utils/gradient.js';
import { shouldUsePnpm, shouldUseYarn } from './utils/packageManager.js';
import {
  frameworkPromptsChoices,
  getSubFrameworkPromptsChoices
} from './utils/prompts.js';

interface IResultType {
  packageName?: string;
  projectName?: string;
  framework?: string;
  argFrameWork?: string;
  autoInstall?: boolean;
  packageManager?: string;
}

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
          choices: frameworkPromptsChoices
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
  const frameworkPrompts = getSubFrameworkPromptsChoices(framework);
  const options = await prompts(frameworkPrompts as any);

  await copyTemplate(targetDir, {
    framework: options.subFramework
      ? `${framework}/${options.subFramework}`
      : framework,
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
