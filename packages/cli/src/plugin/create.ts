import path from 'node:path';
import inquirer from 'inquirer';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';
import { bold, green } from '@farmfe/core';

export interface CreateArgs {
  npmName?: string;
  structName?: string;
  dir?: string;
  language?: 'rust' | 'js';
}

const TEMPLATE_NPM_NAME = '<FARM-RUST-PLUGIN-NPM-NAME>';
const TEMPLATE_JS_NPM_NAME = '<FARM-JS-PLUGIN-NPM-NAME>';
const TEMPLATE_STRUCT_NAME = '<FARM-RUST-PLUGIN-STRUCT-NAME>';

/**
 * Farm plugin create command, create a rust / js farm plugin
 */
export async function create(args: CreateArgs): Promise<void> {
  const prompts = [];
  const commonPrompts = [
    {
      type: 'list',
      name: 'language',
      message: 'What type of plugin do you want to create?',
      choices: [
        {
          name: 'Rust plugin',
          value: 'rust'
        },
        {
          name: 'Javascript plugin',
          value: 'js'
        }
      ]
    }
  ];

  const rustPrompts = [
    {
      type: 'input',
      name: 'dir',
      message: 'Where to create the plugin? E.g. ./farm-plugin-xxx',
      default: './farm-plugin-xxx'
    },
    {
      type: 'input',
      name: 'npmName',
      message: 'What is the npm name of the plugin? E.g. farm-plugin-xxx',
      default: 'farm-plugin-xxx'
    },
    {
      type: 'input',
      name: 'structName',
      message: 'What is the name struct of the plugin? E.g. FarmPluginXxx',
      default: 'FarmPluginXxx'
    }
  ];

  const jsPrompts = [
    {
      type: 'input',
      name: 'dir',
      message: 'Where to create the plugin? E.g. ./farm-js-plugin-xxx',
      default: './farm-js-plugin-xxx'
    },
    {
      type: 'input',
      name: 'npmName',
      message: 'What is the npm name of the plugin? E.g. farm-js-plugin-xxx',
      default: 'farm-js-plugin-xxx'
    }
  ];

  if (!args.language) {
    if (!args.language) {
      const languageChoice = await inquirer.prompt(commonPrompts);
      args.language = languageChoice.language;
    }
  }
  const CHOOSE_PROMPTS = new Map([
    ['rust', rustPrompts],
    ['js', jsPrompts]
  ]);
  const selectedPrompts = CHOOSE_PROMPTS.get(args.language);

  for (const prompt of selectedPrompts) {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (!args[prompt.name]) {
      prompts.push(prompt);
    }
  }

  const ans = await inquirer.prompt(prompts);
  const dir = args.dir || ans.dir;
  const npmName = args.npmName || ans.npmName;
  const structName = args.structName || ans.nameStruct;

  const dest = path.join(process.cwd(), dir);
  const language = args.language || ans.language;
  const TEMPLATE_PLUGIN = new Map([
    ['rust', path.join(TEMPLATES_DIR, 'rust-plugin')],
    ['js', path.join(TEMPLATES_DIR, 'js-plugin')]
  ]);
  copyFiles(TEMPLATE_PLUGIN.get(language), dest, (content) => {
    const rustContent = content
      .replaceAll(new RegExp(TEMPLATE_NPM_NAME, 'g'), npmName)
      .replaceAll(new RegExp(TEMPLATE_STRUCT_NAME, 'g'), structName);
    const jsContent = content.replaceAll(
      new RegExp(TEMPLATE_JS_NPM_NAME, 'g'),
      npmName
    );

    const TEMPLATE_PLUGIN = new Map([
      ['rust', rustContent],
      ['js', jsContent]
    ]);
    return TEMPLATE_PLUGIN.get(language);
  });

  console.log(bold(green(`Plugin created successfully in ${dest}`)));
}
