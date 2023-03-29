#!/usr/bin/env node

import chalk from 'chalk';
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';
import { judgeNodeVersion } from './utils/Environment';
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
        type: framework && TEMPLATES.includes(framework) ? null : 'select',
        name: 'framework',
        message:
          typeof framework === 'string' && !TEMPLATES.includes(framework)
            ? `"${framework}" isn't a valid template. Please choose from below: `
            : 'Select a framework:',
        initial: 0,
        choices: [
          { title: chalk.magenta('Vue'), value: 'vue' },
          {
            title: chalk.blue('React'),
            value: 'react',
          },
        ],
      },
    ]);
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

createFarm();
