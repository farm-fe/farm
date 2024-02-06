import { execSync } from 'child_process';

import { cac } from 'cac';
import { resolveNapiRsCli } from './resolve-napi-rs-cli.js';

import { copyArtifacts } from './copy-artifacts.js';

const getArg = (argsObj: any, args: string[], name: string) => {
  const index = args.indexOf(name);

  if (index === -1) {
    return;
  }
  // get argsObj['abi']
  const value = argsObj[name.slice(2)];

  if (typeof value === 'string') {
    const res = args[index + 1];
    args.splice(index, 2);
    return res;
  } else {
    throw new Error(`Missing ${name}`);
  }
};

const cli = cac('farm-plugin-tools');

cli
  .command('build', 'Build Farm Rust Plugin for current platform')
  .allowUnknownOptions()
  .action((argsObj) => {
    const cliPath = resolveNapiRsCli();
    console.log(cliPath);
    const args = process.argv.slice(3);

    const abi = getArg(argsObj, args, '--abi');

    // all args are passed to napi-rs directly
    // 1. build with napi-rs
    try {
      execSync(`node ${cliPath} build ${args.join(' ')}`, {
        stdio: 'inherit'
      });
    } catch (e) {
      process.exit(1);
    }
    // 2. copy the output to the correct place and rename it to index.farm
    copyArtifacts(abi);
  });

cli
  .command(
    'prepublish',
    'Publish platform packages before publish your Rust Plugin'
  )
  .allowUnknownOptions()
  .action(() => {
    const cliPath = resolveNapiRsCli();
    // just call napi prepubish -t npm
    try {
      execSync(`node ${cliPath} prepublish -t npm`, {
        stdio: 'inherit'
      });
    } catch (e) {
      process.exit(1);
    }
  });

cli.parse();
