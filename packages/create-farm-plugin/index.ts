#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import minimist from 'minimist';
import prompts from 'prompts';

import { fileURLToPath } from 'node:url';
import { colors } from '@farmfe/utils/colors';

interface IResultType {
  pluginName?: string;
  type?: 'js' | 'rust';
  targetDir?: string;
}
// judge node version
judgeNodeVersion();

// command
welcome();

// argv
const argv = minimist<{
  t?: 'js' | 'rust';
  type?: 'js' | 'rust';
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
            { title: colors.cyan('JS Plugin'), value: 'js' },
            { title: colors.green('Rust Plugin'), value: 'rust' }
          ],
          onState: (state) => {
            if (state.value === 'js') {
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
  const { type = argTemplate, pluginName: finalPluginName = argPluginName } =
    result;

  await copyTemplate(targetDir, { type, pluginName: finalPluginName });
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
  copy(templatePath, dest, options);

  // copy .gitignore file if exists
  const gitignore = path.join(templatePath, '.gitignore');
  if (fs.existsSync(gitignore)) {
    fs.copyFileSync(gitignore, path.join(dest, '.gitignore'));
  }

  // Modify package.json to add dependencies
  const packageJsonPath = path.join(`${dest}/playground`, 'package.json');
  if (fs.existsSync(packageJsonPath)) {
    const packageJsonContent = JSON.parse(
      fs.readFileSync(packageJsonPath, 'utf-8')
    );
    // Modify the dependencies object as needed
    packageJsonContent.dependencies[options.pluginName] = 'workspace:*'; // Modify this line with your dependency and version
    fs.writeFileSync(
      packageJsonPath,
      JSON.stringify(packageJsonContent, null, 2)
    );
  }

  const runText = options.type === 'js' ? 'pnpm dev' : 'pnpm build';
  console.log(colors.green('\n🎉 Plugin created successfully!\n'));
  console.log(colors.cyan(`cd ${targetDir} && pnpm install && ${runText}\n`));
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

function replaceNamePlaceholders(
  content: string,
  options: IResultType
): string {
  const PLACEHOLDERS = [
    {
      name: '<FARM-JS-PLUGIN-NPM-NAME>',
      replace: () => options.pluginName
    },
    {
      name: '<FARM-RUST-PLUGIN-NPM-NAME>',
      replace: () => options.pluginName
    },
    {
      name: '<FARM-RUST-PLUGIN-CARGO-NAME>',
      replace: () => {
        // replace @ to empty string and all invalid characters to _
        return options.pluginName
          .replace(/@/g, '')
          .replace(/[^a-zA-Z0-9_]/g, '_');
      }
    },
    {
      name: '<FARM-RUST-PLUGIN-STRUCT-NAME>',
      replace: () => {
        return (
          options.pluginName
            .replace(/@/g, '')
            .replace(/[^a-zA-Z0-9_]/g, '_')
            // to camelCase
            .replace(/_([a-z])/g, (g) => g[1].toUpperCase())
            // replace first character to upper case
            .replace(/^[a-z]/, (g) => g.toUpperCase())
        );
      }
    }
  ];

  for (const placeholder of PLACEHOLDERS) {
    if (content.includes(placeholder.name)) {
      content = content.replaceAll(placeholder.name, placeholder.replace());
    }
  }

  return content;
}

function copy(src: string, dest: string, options: IResultType) {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    copyDir(src, dest, options);
  } else {
    fs.copyFileSync(src, dest);
    const destContent = fs.readFileSync(dest, 'utf-8');
    fs.writeFileSync(dest, replaceNamePlaceholders(destContent, options));
  }
}

function copyDir(srcDir: string, destDir: string, options: IResultType) {
  fs.mkdirSync(destDir, { recursive: true });
  for (const file of fs.readdirSync(srcDir)) {
    const srcFile = path.resolve(srcDir, file);
    const destFile = path.resolve(destDir, file);
    if (file === 'gitignore') {
      copy(srcFile, destFile, options);
      fs.renameSync(destFile, path.resolve(destDir, '.gitignore'));
    } else {
      copy(srcFile, destFile, options);
    }
  }
}

function welcome() {
  console.log(colors.BrandText('⚡ Welcome To Farm ! '));
}

createFarm();
