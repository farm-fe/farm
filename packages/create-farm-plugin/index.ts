#!/usr/bin/env node
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';

import { fileURLToPath } from 'node:url';
import { colors } from '@farmfe/utils/colors';

interface IResultType {
  pluginName?: string;
  type?: 'js' | 'rust';
}
// judge node version
judgeNodeVersion();

// command
welcome();

// argv
const argv = minimist<{
  t?: string;
  type?: string;
}>(process.argv.slice(2), { string: ['_'] });

const cwd = process.cwd();

async function createFarm() {
  const argPluginName = formatTargetDir(argv._[0]);
  const argTemplate = argv.type || argv.t;
  let targetDir = argPluginName;
  let result: IResultType = {};
  let pluginName = argPluginName || 'farm-plugin-xxx';

  try {
    result = await prompts(
      [
        {
          type: argTemplate ? null : 'select',
          name: 'type',
          message: 'Select Plugin Type:',
          choices: [
            { title: colors.cyan('JS Plugin'), value: 'js-plugin' },
            { title: colors.green('Rust Plugin'), value: 'rust-plugin' }
          ],
          onState: (state) => {
            if (state.value === 'js-plugin') {
              pluginName = 'farm-js-plugin-xxx';
            }
          }
        },
        {
          type: argPluginName ? null : 'text',
          name: 'pluginName',
          message: 'Plugin Name:',
          initial: () => pluginName,
          validate: (name) => {
            if (name.trim().length === 0) {
              return 'Plugin name cannot be empty';
            }
            return true;
          },
          onState: (state) => {
            targetDir = formatTargetDir(state.value) || pluginName;
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
  const { type } = result;

  await copyTemplate(targetDir, { type, pluginName: targetDir });
}

function formatTargetDir(targetDir: string | undefined) {
  return targetDir?.trim().replace(/\/+$/g, '');
}

function isEmpty(path: string) {
  const files = fs.readdirSync(path);
  return files.length === 0 || (files.length === 1 && files[0] === '.git');
}

async function copyTemplate(targetDir: string, options: IResultType) {
  const dest = path.join(cwd, targetDir);
  const templatePath = path.join(
    fileURLToPath(import.meta.url),
    `../../templates/${options.type}`
  );
  copy(templatePath, dest);

  writePackageJson(dest, options);

  console.log(colors.green('\n🎉 Plugin created successfully!\n'));
  console.log(colors.cyan(`cd ${targetDir} && pnpm install && pnpm dev\n`));
}

function writePackageJson(dest: string, options: IResultType) {
  const pkg = JSON.parse(
    fs.readFileSync(path.join(dest, `package.json`), 'utf-8')
  );

  pkg.name = options.pluginName;

  const packageJsonPath = path.join(dest, 'package.json');
  const { name, ...rest } = pkg;
  const sortedPackageJson = { name, ...rest };
  fs.writeFileSync(
    packageJsonPath,
    JSON.stringify(sortedPackageJson, null, 2) + '\n'
  );
}

function judgeNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;
  if (requiredMajorVersion < minimumMajorVersion) {
    console.log(
      colors.yellow(
        `create-farm-plugin unsupported Node.js v${currentVersion}.`
      )
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
