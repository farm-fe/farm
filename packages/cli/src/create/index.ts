import path from 'node:path';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';
import inquirer from 'inquirer';
import chalk from 'chalk';

const TEMPLATE_REACT = path.join(TEMPLATES_DIR, 'react');
const TEMPLATE_VUE = path.join(TEMPLATES_DIR, 'vue');
const REACT = 'React';
const VUE = 'Vue';

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
  const dest = path.join(process.cwd(), projectName);
  if (framework === REACT) {
    copyFiles(TEMPLATE_REACT, dest);
    logger(dest, projectName);
  } else if (framework === VUE) {
    copyFiles(TEMPLATE_VUE, dest);
    logger(dest, projectName);
  } else {
    throw new Error(`Please choose legal template!`);
  }
}

function logger(dest: string, projectName: string) {
  console.log(
    chalk.green('Created a new Farm app in ') +
      chalk.bold(dest) +
      chalk.green('.')
  );
  console.log(
    `Run ${chalk.cyan.bold(
      `cd ${projectName} && npm i && npm start`
    )} to get started.`
  );
}
