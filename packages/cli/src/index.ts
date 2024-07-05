import { readFileSync } from 'node:fs';

import { defineCommand, runMain } from 'citty';

import buildCommand from './commands/build.js';
import cleanCommand from './commands/clean.js';
import devCommand from './commands/dev.js';
import previewCommand from './commands/preview.js';
import watchCommand from './commands/watch.js';

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);

const main = defineCommand({
  meta: {
    name: 'farm',
    version
  },
  args: {
    config: {
      type: 'string',
      alias: 'c',
      description: 'use specified config file'
    },
    mode: { type: 'string', alias: 'm', description: 'set env mode' },
    base: { type: 'string', description: 'public base path' },
    clearScreen: {
      type: 'boolean',
      default: true,
      description: 'allow/disable clear screen when logging'
    }
  },
  subCommands: {
    dev: devCommand,
    // alias for dev
    start: devCommand,
    build: buildCommand,
    watch: watchCommand,
    preview: previewCommand,
    clean: cleanCommand
  }
});

// default to start a development server
if (process.argv.slice(2).length === 0)
  runMain(main, { rawArgs: process.argv.slice(2).concat(['dev']) });
else runMain(main);
