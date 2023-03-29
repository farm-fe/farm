#!/usr/bin/env node

import chalk from 'chalk';
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';

import { loadWithRocketGradient } from './utils/gradient';
import createSpawnCmd from './utils/createSpawnCmd';

// judge node version
judgeNodeVersion();

// command
console.log(chalk.magenta(`\n⚡ Welcome To Create Farm Project!`));
console.log();
const argv = minimist<{
  t?: string;
  template?: string;
}>(process.argv.slice(2), { string: ['_'] });
const cwd = process.cwd();

const DEFAULT_TARGET_NAME = 'farm-project';

async function createFarm() {
  const projectName = formatTargetDir(argv._[0]);
  const framework = argv.template || argv.t;
  let targetDir = projectName || DEFAULT_TARGET_NAME;
  const getProjectName = () =>
    targetDir === '.' ? path.basename(path.resolve()) : targetDir;
  let result = null;
  let frameworkName = null;
  let installFlag = false;
  try {
    result = await prompts([
      {
        type: projectName ? null : 'text',
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
        type: framework ? null : 'select',
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
        onState: (state: any) => {
          frameworkName = state.value;
        },
      },
      {
        type: 'confirm',
        name: 'install',
        message: 'Whether you need to install dependencies automatically ?',
        onState: (state: any) => {
          installFlag = state.value;
        },
      },
    ]);
    await copyTemplate(targetDir, frameworkName ?? framework);
    installFlag && (await installationDeps(targetDir));
  } catch (cancelled: any) {
    console.log(cancelled.message);
    return;
  }
}

function formatTargetDir(targetDir: string | undefined) {
  return targetDir?.trim().replace(/\/+$/g, '');
}

function isEmpty(path: string) {
  const files = fs.readdirSync(path);
  return files.length === 0 || (files.length === 1 && files[0] === '.git');
}

function emptyDir(dir: string) {
  if (!fs.existsSync(dir)) {
    return;
  }
  for (const file of fs.readdirSync(dir)) {
    if (file === '.git') {
      continue;
    }
    fs.rmSync(path.resolve(dir, file), { recursive: true, force: true });
  }
}

async function copyTemplate(targetDir: string, framework: string) {
  const spinner = await loadWithRocketGradient('copy template');
  const dest = path.join(cwd, targetDir);
  const templatePath = path.join(__dirname, `../templates/${framework}`);
  copy(templatePath, dest);
  spinner.text = 'Template copied!';
  spinner.succeed();
}

async function installationDeps(targetDir: string) {
  const cmdInherit = createSpawnCmd(path.resolve(cwd, targetDir));
  const startTime: number = new Date().getTime();
  const pkgInfo = pkgFromUserAgent(process.env.npm_config_user_agent);
  const pkgManager = pkgInfo ? pkgInfo.name : 'npm';
  await cmdInherit(pkgManager, ['install']);
  const endTime: number = new Date().getTime();
  const usageTime: number = (endTime - startTime) / 1000;
  logger('> Initial Farm Project created successfully');
  logger(
    `> Usage time ${usageTime}s , Please enter the following command to continue...`
  );
  logger(`  cd ${targetDir}`);
  logger(`  ${pkgManager} ${pkgManager === 'npm' ? 'run' : ''} start`);
}

function logger(info: string) {
  console.log();
  console.log(chalk.magenta(info));
}

function sleep(num: number) {
  return new Promise((resolve) =>
    setTimeout(() => {
      resolve(123);
    }, num)
  );
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
