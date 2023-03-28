import path from 'node:path';
import fs from 'node:fs';
import {
  copyFiles,
  formatTargetDir,
  install,
  TEMPLATES_DIR,
} from '../utils.js';
import inquirer from 'inquirer';
import chalk from 'chalk';

const TEMPLATE_REACT = path.join(TEMPLATES_DIR, 'react');
const TEMPLATE_VUE = path.join(TEMPLATES_DIR, 'vue');
const REACT = 'React';
const VUE = 'Vue';
const NPM = 'npm';
const PNPM = 'pnpm';
const YARN = 'yarn';

export async function create(defaultProjectName: string): Promise<void> {
  let projectName = formatTargetDir(defaultProjectName);
  if (!projectName) {
    await inquirer
      .prompt({
        type: 'input',
        name: 'name',
        message: 'please input project name',
        default: 'my-farm-project',
      })
      .then(async (answer) => {
        projectName = formatTargetDir(answer.name);
      });
  }
  let root = path.resolve(process.cwd(), projectName);
  while (fs.existsSync(root)) {
    console.log(
      chalk.redBright.bold(
        `${projectName} is not empty, please choose another project name`
      )
    );
    await inquirer
      .prompt({
        type: 'input',
        name: 'name',
        message: 'please input project name',
        default: 'my-farm-project',
      })
      .then(async (answer) => {
        projectName = answer.name;
        root = path.resolve(process.cwd(), projectName);
      });
  }
  //   while(fs.existsSync(root)) {
  //
  //     inquirer.prompt({
  //     type: 'input',
  //     name: 'name',
  //     message: 'please input project name',
  //     default: 'my-farm-project',
  //   }).then((answer) => {
  //       projectName = answer.name;
  //       root = path.resolve(process.cwd(), projectName);
  //   });
  // }
  // });
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
    await install({
      cwd: root,
      package: pkgManager,
    });
    logger(dest, projectName, pkgManager);
  } else if (framework === VUE) {
    copyFiles(TEMPLATE_VUE, dest);
    await install({
      cwd: root,
      package: pkgManager,
    });
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
    `Run  \n ${chalk.cyan.bold(
      `cd ${projectName}\n ${pkgManager} start`
    )} \n to get started.`
  );
}
