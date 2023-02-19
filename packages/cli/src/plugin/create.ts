import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';
import walk from 'walkdir';
import inquirer from 'inquirer';
import chalk from 'chalk';
import { fileURLToPath } from 'url';

export interface CreateArgs {
  npmName?: string;
  structName?: string;
  dir?: string;
}

const TEMPLATES_DIR = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../templates/rust-plugin'
);
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

  walk(TEMPLATES_DIR, { sync: true }, (p, stat) => {
    if (stat.isFile()) {
      const content = readFileSync(p).toString();
      const newContent = content
        .replace(new RegExp(TEMPLATE_NPM_NAME, 'g'), npmName)
        .replace(new RegExp(TEMPLATE_STRUCT_NAME, 'g'), structName);

      const relativePath = path.relative(TEMPLATES_DIR, p);
      const destPath = path.join(dest, relativePath);

      if (!existsSync(path.dirname(destPath))) {
        mkdirSync(path.dirname(destPath), { recursive: true });
      }

      writeFileSync(destPath, newContent);
    }
  });

  console.log(chalk.green(`Plugin created successfully in ${dest}`));
}
