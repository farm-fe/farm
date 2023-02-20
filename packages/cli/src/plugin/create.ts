import path from 'path';
import inquirer from 'inquirer';
import chalk from 'chalk';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';

export interface CreateArgs {
  npmName?: string;
  structName?: string;
  dir?: string;
}

const TEMPLATE_PLUGIN = path.join(TEMPLATES_DIR, 'rust-plugin');
const TEMPLATE_NPM_NAME = '<FARM-RUST-PLUGIN-NPM-NAME>';
const TEMPLATE_STRUCT_NAME = '<FARM-RUST-PLUGIN-STRUCT-NAME>';

/**
 * Farm plugin create command, create a rust farm plugin
 */
export async function create(args: CreateArgs): Promise<void> {
  const prompts = [];

  if (!args.dir) {
    prompts.push({
      type: 'input',
      name: 'dir',
      message: 'Where to create the plugin? E.g. ./farm-plugin-xxx',
      default: '.',
    });
  }

  if (!args.npmName) {
    prompts.push({
      type: 'input',
      name: 'npmName',
      message: 'What is the npm name of the plugin? E.g. farmfe-plugin-xxx',
    });
  }

  if (!args.structName) {
    prompts.push({
      type: 'input',
      name: 'structName',
      message: 'What is the name struct of the plugin? E.g. FarmPluginXxx',
    });
  }

  const ans = await inquirer.prompt(prompts);
  const dir = args.dir || ans.dir;
  const npmName = args.npmName || ans.npmName;
  const structName = args.structName || ans.nameStruct;

  const dest = path.join(process.cwd(), dir);

  copyFiles(TEMPLATE_PLUGIN, dest, (content) => {
    return content
      .replace(new RegExp(TEMPLATE_NPM_NAME, 'g'), npmName)
      .replace(new RegExp(TEMPLATE_STRUCT_NAME, 'g'), structName);
  });

  console.log(chalk.green(`Plugin created successfully in ${dest}`));
}
