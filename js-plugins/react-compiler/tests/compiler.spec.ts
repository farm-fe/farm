import { expect, test } from 'vitest';
import { reactCompiler } from '../src/index';

test('TypeScript syntax outside compiled components is removed from emitted content', async () => {
  const plugin = reactCompiler();
  const result = await plugin.processModule.executor({
    moduleId: '/fixture/main.tsx',
    moduleType: 'tsx',
    content:
      'const root: HTMLElement = document.querySelector("#app")!;root.setAttribute("data-ready","yes");export const element = <button>Ready</button>;'
  });
  expect(result.content).not.toContain(': HTMLElement');
  expect(result.content).not.toMatch(/querySelector\([^)]*\)!/);
});

const jsx =
  'export function Card({label}) { return <button>{label}</button>; }';
test('default TSX filter accepts TypeScript annotations', async () => {
  const plugin = reactCompiler();
  const result = await plugin.processModule.executor({
    moduleId: '/fixture/Card.tsx',
    moduleType: 'tsx',
    content:
      'export function Card({label}: {label: string}) { return <button>{label}</button>; }'
  });
  expect(result.content).toContain('react/compiler-runtime');
});
test('compiler target is forwarded to React Compiler', async () => {
  const plugin = reactCompiler({ compilerOptions: { target: '18' } });
  const result = await plugin.processModule.executor({
    moduleId: '/fixture/Card.jsx',
    moduleType: 'jsx',
    content: jsx
  });
  expect(result.content).toContain('react-compiler-runtime');
  expect(result.content).not.toContain('react/compiler-runtime');
});
test('caller Babel transform options are respected', async () => {
  const plugin = reactCompiler({ transformOptions: { comments: false } });
  const result = await plugin.processModule.executor({
    moduleId: '/fixture/Card.jsx',
    moduleType: 'jsx',
    content: '/* private hint */' + jsx
  });
  expect(result.content).not.toContain('private hint');
});
