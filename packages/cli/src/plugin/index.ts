/**
 * plugin subcommand:
 * * create: create a new farm plugin
 * * build: build a new farm plugin, support cross compilation natively
 */

import { build } from './build';
import { create } from './create';

export const COMMANDS = {
  build,
  create,
};
