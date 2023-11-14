import { execa } from 'execa';
import os from 'node:os';
import {
  DEFAULT_PACKAGE_MANAGER,
  buildExamples,
  runTaskQueue
} from './build.mjs';

const cwd = process.cwd();

console.log('Installing dependencies...');
await execa(DEFAULT_PACKAGE_MANAGER, ['install'], { cwd });

console.log('Cleaning...');
await execa('node', ['./scripts/clean.mjs'], { cwd });

console.log('Code Spell lint...');
await execa('npx', ['cspell', '**', '--gitignore'], { cwd });

console.log('Cargo check');
await execa('cargo', ['check', '--color', 'always', '--all', '--all-targets'], {
  cwd
});

console.log('Cargo clippy');
await execa('cargo', ['clippy'], { cwd });

console.log('TypeScript check');
await execa(
  DEFAULT_PACKAGE_MANAGER,
  ['run', '--filter', '"@farmfe/*"', 'type-check'],
  { cwd }
);

console.log('Cargo test');
await execa(
  'cargo',
  // When there are too many jobs, Out of Memory may appear
  ['test', '-j', Math.max(Math.floor(os.cpus().length / 4), 1)],
  { cwd }
);

console.log('build core、js/rust plugins、cli ...');
await runTaskQueue();

console.log('Building core CJS...');
await buildCoreCjs();

console.log('build examples');
await buildExamples();
