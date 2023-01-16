/**
 * plugin subcommand:
 * * create: create a new farm plugin
 * * build: build a new farm plugin, support cross compilation natively
 */

import { build } from './build.js';
// import { create } from './create.js';

export const COMMANDS = {
  build,
  // create,
};
