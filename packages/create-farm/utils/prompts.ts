import { colors } from './color.js';

const allFramework = new Map([
  ['tauri', ['lit', 'tauri', 'electron', 'vue2']],
  ['ssr', ['lit', 'tauri', 'electron']],
  ['electron', ['lit', 'tauri', 'electron']]
]);

export const frameworkPromptsChoices = [
  {
    title: colors.cyan('React'),
    value: 'react'
  },
  { title: colors.green('Vue'), value: 'vue' },
  { title: colors.bgGreen('Vue2'), value: 'vue2' },
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
  const choices = frameworkPromptsChoices.filter(
    (item) => !filterFramework?.includes(item.value)
  );

  return {
    type: filterFramework ? 'select' : null,
    name: 'subFramework',
    message: 'Select a framework:',
    choices
  };
}
