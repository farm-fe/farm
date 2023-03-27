import path from 'node:path';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';
import inquirer from 'inquirer';
import chalk from 'chalk';

const TEMPLATE_REACT = path.join(TEMPLATES_DIR, 'react');
const TEMPLATE_VUE = path.join(TEMPLATES_DIR, 'vue');
const REACT = 'React';
const VUE = 'Vue';
const NPM = 'npm';
const PNPM = 'pnpm';
const YARN = 'yarn';

export async function create(): Promise<void> {
  const { name: projectName } = await inquirer.prompt({
    type: 'input',
    name: 'name',
    message: 'please input project name',
    default: 'my-farm-project',
  });

  const { framework } = await inquirer.prompt({
    type: 'list',
    name: 'framework',
    message: 'please choose a framework',
    choices: [
      {
        name: '1) React',
        value: REACT,
      },
      {
        name: '2) Vue',
        value: VUE,
      },
    ],
  });
  const { pkgManager } = await inquirer.prompt({
    type: 'list',
    name: 'pkgManager',
    message: 'please choose your package Manager',
    choices: [
      {
        name: '1) npm',
        value: NPM,
      },
      {
        name: '2) pnpm',
        value: PNPM,
      },
      {
        name: '3) yarn',
        value: YARN,
      },
    ],
  });
  const dest = path.join(process.cwd(), projectName);
  if (framework === REACT) {
    copyFiles(TEMPLATE_REACT, dest);
    logger(dest, projectName, pkgManager);
  } else if (framework === VUE) {
    copyFiles(TEMPLATE_VUE, dest);
    logger(dest, projectName, pkgManager);
  } else {
    throw new Error(`Please choose legal template!`);
  }
}

function logger(dest: string, projectName: string, pkgManager: string) {
  console.log(
    chalk.green('Created a new Farm app in ') +
      chalk.bold(dest) +
      chalk.green('.')
  );
  console.log(
    `Run ${chalk.cyan.bold(
      `cd ${projectName} && ${pkgManager} i && ${pkgManager} start`
    )} to get started.`
  );
}
