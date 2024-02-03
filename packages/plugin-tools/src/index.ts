import { execSync } from 'child_process';

import { cac } from 'cac';
import { resolveNapiRsCli } from './resolve-napi-rs-cli.js';

const cli = cac('farm-plugin-tools');

cli
  .command('build', 'Build Farm Rust Plugin for current platform')
  .action(() => {
    console.log('Building...');
    const cliPath = resolveNapiRsCli();
    console.log(cliPath);

    // 1. build with napi-rs

    // 2. copy the output to the correct place and rename it to index.farm
  });

cli
  .command(
    'prepublish',
    'Publish platform packages before publish your Rust Plugin'
  )
  .action(() => {
    console.log('Preparing...');
    const cliPath = resolveNapiRsCli();
    // just call napi prepubish -t npm
  });

cli.parse();
