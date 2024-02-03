/**
 * plugin subcommand:
 * * create: create a new farm plugin
 * * build: build a new farm plugin, support cross compilation natively
 */

// import { build } from './build.js';
// import { create } from './create.js';
// import { prepublish } from './prepublish.js';

export const COMMANDS = {
  // build,
  create: async (_: unknown) => {
    const { colors } = await import('@farmfe/core');
    console.log(
      colors.red(
        'farm plugin create is deprecated, please use `pnpm create farm-plugin` instead.'
      )
    );
    process.exit(1);
  },
  prepublish: async (_: unknown) => {
    const { colors } = await import('@farmfe/core');
    console.log(
      colors.red(
        'farm plugin prepublish is deprecated, please use `pnpm install -D @farmfe/plugin-tools && farm-plugin-tools prepublish` instead.'
      )
    );
    process.exit(1);
  }
};
