import { colors } from './color.js';

const allFramework = new Map([
  ['', []],
  ['ssr', ['lit']],
  ['tauri', ['lit']],
  ['electron', ['lit']]
]);

export const frameworkPromptsChoices = [
  { title: colors.green('Web'), value: '' },
  // { title: colors.green('Ssr'), value: 'ssr' },
  {
    title: colors.cyan('tauri'),
    value: 'tauri'
  }
  // { title: colors.orange('electron'), value: 'electron' },
];

export const subFrameworkPromptsChoices = [
  {
    title: colors.cyan('React'),
    value: 'react'
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
  { title: colors.red('Lit'), value: 'lit' },
  { title: colors.orange('Tauri'), value: 'tauri' }
];

export function getSubFrameworkPromptsChoices(framework: string) {
  const filterFramework = allFramework.get(framework);
  const choices = subFrameworkPromptsChoices.filter(
    (item) => !filterFramework?.includes(item.value)
  );
  return {
    type: 'select',
    name: 'subFramework',
    message: 'Select a framework:',
    choices
  };
}
