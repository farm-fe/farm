import { createSpawnCmd } from '@farmfe/utils';
import { cac } from 'cac';
import { copyArtifacts } from './copy-artifacts.js';
import { prepublish } from './prepublish.js';
import { resolveNapiRsCli } from './resolve-napi-rs-cli.js';

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
  .action(async (argsObj) => {
    const cliPath = resolveNapiRsCli();
    const args = process.argv.slice(3);
    const abi = getArg(argsObj, args, '--abi');

    // all args are passed to napi-rs directly
    // 1. build with napi-rs
    try {
      const spawn = createSpawnCmd(process.cwd(), 'inherit');
      await spawn('node', [cliPath, 'build', ...args]);
    } catch {
      process.exit(1);
    }

    // wait 500ms for the output to be ready
    await new Promise((resolve) => setTimeout(resolve, 500));

    // 2. copy the output to the correct place and rename it to index.farm
    copyArtifacts(abi);
  });

cli
  .command(
    'prepublish',
    'Publish platform packages before publish your Rust Plugin'
  )
  .allowUnknownOptions()
  .action(async () => {
    await prepublish();
  });

cli.parse();
