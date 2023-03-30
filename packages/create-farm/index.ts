#!/usr/bin/env node

import chalk from 'chalk';
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';

import { loadWithRocketGradient } from './utils/gradient';
import createSpawnCmd from './utils/createSpawnCmd';

interface IResultType {
  packageName?: string;
  framework?: string;
  autoInstall?: boolean;
}

// judge node version
judgeNodeVersion();

// command
console.log(chalk.magenta(`\n⚡ Welcome To Create Farm Project!`));
console.log();

// argv
const argv = minimist<{
  t?: string;
  template?: string;
}>(process.argv.slice(2), { string: ['_'] });

const cwd = process.cwd();

const DEFAULT_TARGET_NAME = 'farm-project';

async function createFarm() {
  const argProjectName = formatTargetDir(argv._[0]);
  const argFramework = argv.template || argv.t;
  let targetDir = argProjectName || DEFAULT_TARGET_NAME;
  let result: IResultType = {};
  try {
    result = await prompts(
      [
        {
          type: argProjectName ? null : 'text',
          name: 'projectName',
          message: 'Project name:',
          initial: DEFAULT_TARGET_NAME,
          onState: (state: any) => {
            targetDir = formatTargetDir(state.value) || DEFAULT_TARGET_NAME;
          },
        },
        {
          type: () =>
            !fs.existsSync(targetDir) || isEmpty(targetDir) ? null : 'confirm',
          name: 'overwrite',
          message: () =>
            (targetDir === '.'
              ? '🚨 Current directory'
              : `🚨 Target directory "${targetDir}"`) +
            ` is not empty. Remove existing files and continue?`,
        },
        {
          type: (_: any, { overwrite }: { overwrite?: boolean }): any => {
            if (overwrite === false) {
              throw new Error(chalk.red('❌') + ' Operation cancelled');
            }
            return null;
          },
          name: 'overwriteChecker',
        },
        {
          type: argFramework ? null : 'select',
          name: 'framework',
          message: 'Select a framework:',
          initial: 0,
          choices: [
            { title: chalk.green('Vue'), value: 'vue' },
            {
              title: chalk.blue('React'),
              value: 'react',
            },
          ],
        },
        {
          type: 'confirm',
          name: 'autoInstall',
          message: 'Whether you need to install dependencies automatically ?',
        },
      ],
      {
        onCancel: () => {
          throw new Error(chalk.red('❌') + ' Operation cancelled');
        },
      }
    );
  } catch (cancelled: any) {
    console.log(cancelled.message);
    process.exit(1);
  }
  const { framework = argFramework, autoInstall } = result;

  await copyTemplate(targetDir, framework);
  await installationDeps(targetDir, autoInstall!);
}

function formatTargetDir(targetDir: string | undefined) {
  return targetDir?.trim().replace(/\/+$/g, '');
}

function isEmpty(path: string) {
  const files = fs.readdirSync(path);
  return files.length === 0 || (files.length === 1 && files[0] === '.git');
}

async function copyTemplate(targetDir: string, framework: string) {
  const spinner = await loadWithRocketGradient('copy template');
  const dest = path.join(cwd, targetDir);
  const templatePath = path.join(__dirname, `../templates/${framework}`);
  copy(templatePath, dest);
  spinner.text = 'Template copied!';
  spinner.succeed();
}

async function installationDeps(targetDir: string, autoInstall: boolean) {
  const pkgInfo = pkgFromUserAgent(process.env.npm_config_user_agent);
  const pkgManager = pkgInfo ? pkgInfo.name : 'npm';
  if (autoInstall) {
    const cmdInherit = createSpawnCmd(path.resolve(cwd, targetDir));
    await cmdInherit(pkgManager, ['install']);
  }
  logger('> Initial Farm Project created successfully');
  logger(`  cd ${targetDir}`);
  logger(`  ${pkgManager} ${pkgManager === 'npm' ? 'run' : ''} start`);
}

function logger(info: string) {
  console.log();
  console.log(chalk.magenta(info));
}

function pkgFromUserAgent(userAgent: string | undefined) {
  if (!userAgent) return undefined;
  const pkgSpec = userAgent.split(' ')[0];
  const pkgSpecArr = pkgSpec.split('/');
  return {
    name: pkgSpecArr[0],
    version: pkgSpecArr[1],
  };
}

function judgeNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;

  if (requiredMajorVersion < minimumMajorVersion) {
    console.log(
      chalk.yellow(`create-farm unsupported Node.js v${currentVersion}.`)
    );
    console.log(
      chalk.yellow(`Please use Node.js v${minimumMajorVersion} or higher.`)
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
    copy(srcFile, destFile);
  }
}

createFarm();
