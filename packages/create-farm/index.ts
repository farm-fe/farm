#!/usr/bin/env node
import prompts from 'prompts';
import minimist from 'minimist';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

import ejs from 'ejs';
import { colors, reset } from './utils/color.js';

import { loadWithRocketGradient } from './utils/gradient.js';
import { getLanguage } from './utils/getLanguage.js';
import createSpawnCmd from './utils/createSpawnCmd.js';
import { shouldUseYarn, shouldUsePnpm } from './utils/packageManager.js';
import renderTemplate from './utils/renderTemplate.js';
import { preOrderDirectoryTraverse } from './utils/directoryTraverse.js';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
interface IResultType {
  packageName?: string;
  projectName?: string;
  framework?: Framework;
  needsTypeScript?: boolean;
  needsSass?: boolean;
  useCssPreProcessor?: string;
  variant?: string;
  argFrameWork?: string;
  autoInstall?: boolean;
  packageManager?: string;
}

type Framework = {
  value: string;
  title: string;
  variants?: FrameworkVariant[];
};
type FrameworkVariant = {
  value: string;
  title: string;
  customCommand?: string;
};

const FRAMEWORKS: Framework[] = [
  {
    title: colors.cyan('React'),
    value: 'react',
    variants: [
      {
        title: colors.cyan('React'),
        value: 'react'
      },
      {
        title: colors.cyan('React-SSR'),
        value: 'react-ssr'
      }
    ]
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
  { title: colors.red('Lit'), value: 'lit' }
];

const CSS_PRE_PROCESSOR = [
  { title: colors.sass('Rust-Sass'), value: 'rust-sass' },
  { title: colors.purple('Sass'), value: 'js-sass' },
  { title: colors.less('Less'), value: 'js-less' },
  { title: colors.postcss('PostCSS'), value: 'js-postcss' },
  { title: colors.tailwindcss('Tailwindcss'), value: 'tailwindcss' },
  { title: colors.red('None'), value: 'none' }
];

const TEMPLATES = FRAMEWORKS.map(
  (f) => (f.variants && f.variants.map((v) => v.value)) || [f.value]
).reduce((a, b) => a.concat(b), []);
// judge node version
judgeNodeVersion();

//
const language = getLanguage();

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
          message: language.packageName.message,
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
              ? `🚨 ${language.shouldOverwrite.dirForPrompts.current}`
              : `🚨 ${language.shouldOverwrite.dirForPrompts.target} "${targetDir}"`) +
            ` ${language.shouldOverwrite.message}`
        },
        {
          type: (_, { overwrite }: { overwrite?: boolean }) => {
            if (overwrite === false) {
              throw new Error(
                colors.red('❌') + `${language.errors.operationCancelled}`
              );
            }
            return null;
          },
          name: 'overwriteChecker'
        },
        {
          type: argFramework ? null : 'select',
          name: 'framework',
          message:
            typeof argFramework === 'string' &&
            !TEMPLATES.includes(argFramework)
              ? reset(`"${argFramework}" ${language.validTemplate.message}`)
              : reset(language.selectFramework.message),
          initial: 0,
          choices: FRAMEWORKS.map((framework) => ({
            title: framework.title,
            value: framework
          }))
        },
        {
          type: (framework: Framework) =>
            framework && framework.variants ? 'select' : null,
          name: 'variant',
          message: reset(language.selectVariant.message),
          choices: (framework: Framework) =>
            framework.variants.map((variant) => {
              return {
                title: variant.title,
                value: variant.value
              };
            })
        },
        {
          name: 'needsTypeScript',
          type: (arg: Framework | string) => {
            return arg === 'react' ? 'toggle' : null;
          },
          message: language.needsTypeScript.message,
          initial: false,
          active: language.defaultToggleOptions.active,
          inactive: language.defaultToggleOptions.inactive
        },
        {
          name: 'useCssPreProcessor',
          type: 'select',
          message: language.useCssPreProcessor.message,
          initial: 0,
          choices: CSS_PRE_PROCESSOR
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
          throw new Error(
            colors.red('❌') + ` ${language.errors.operationCancelled}`
          );
        }
      }
    );
  } catch (cancelled) {
    console.log(cancelled.message);
    return;
  }
  const {
    framework = { title: argFramework, value: argFramework },
    packageManager,
    needsSass,
    needsTypeScript,
    variant,
    useCssPreProcessor
  } = result;

  await copyTemplate(targetDir, {
    projectName: targetDir,
    framework,
    packageManager,
    needsSass,
    needsTypeScript,
    variant,
    useCssPreProcessor
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
  const spinner = await loadWithRocketGradient(language.copy.scaffolding);
  const { variant, framework, needsTypeScript, useCssPreProcessor } = options;
  const templateRoot = path.resolve(__dirname, '../templates');
  const callbacks: ((dataStore: any) => Promise<void>)[] = [];
  const root = path.join(cwd, targetDir);

  function render(templateName: string) {
    const templateDir = path.resolve(templateRoot, './' + templateName);
    renderTemplate(templateDir, root, callbacks);
  }
  let template = variant || framework.value;
  if (needsTypeScript !== false) {
    template += '/ts';
  } else {
    template += '/js';
  }
  const dest = path.join(cwd, targetDir);
  render(template);
  render('config/base/react');
  if (useCssPreProcessor !== 'none') {
    render(`config/css-pre-processor/${useCssPreProcessor}`);
  }
  const dataStore = {};
  // Process callbacks
  for (const cb of callbacks) {
    await cb(dataStore);
  }
  preOrderDirectoryTraverse(
    root,
    () => {},
    (filepath) => {
      if (filepath.endsWith('.ejs')) {
        const template = fs.readFileSync(filepath, 'utf-8');
        const dest = filepath.replace(/\.ejs$/, '');
        const content = ejs.render(template, dataStore[dest]);

        fs.writeFileSync(dest, content);
        fs.unlinkSync(filepath);
      }
    }
  );
  if (needsTypeScript !== false) {
    preOrderDirectoryTraverse(
      root,
      () => {},
      (filepath) => {
        if (filepath.endsWith('.js')) {
          const tsFilePath = filepath.replace(/\.js$/, '.ts');
          if (fs.existsSync(tsFilePath)) {
            fs.unlinkSync(filepath);
          } else {
            fs.renameSync(filepath, tsFilePath);
          }
        }
      }
    );
  } else {
    // Remove all the remaining `.ts` files
    preOrderDirectoryTraverse(
      root,
      () => {},
      (filepath) => {
        if (filepath.endsWith('.ts')) {
          fs.unlinkSync(filepath);
        }
      }
    );
  }
  // copy(templatePath, dest);
  writePackageJson(dest, options);
  spinner.text = language.copy.done;
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
    spinner.text = language.infos.done;
    spinner.succeed();
  }
  colors.handleBrandText(`\n > ${language.infos.done} ✨ ✨ \n`);
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
  console.log(colors.BrandText('⚡ Welcome To Farm !'));
}

createFarm();
