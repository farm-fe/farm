import { readFileSync } from 'fs';
import path from 'path';
import walk from 'walkdir';

export interface CreateArgs {
  name: string;
  dir: string;
}

const TEMPLATES_DIR = path.join(__dirname, '../../templates/rust-plugin');
const TEMPLATE_CARGO_NAME = '<FARM-RUST-PLUGIN-CARGO-NAME>';
const TEMPLATE_NPM_NAME = '<FARM-RUST-PLUGIN-NPM-NAME>';
const TEMPLATE_NAME_STRUCT = '<FARM-RUST-PLUGIN-NAME-STRUCT>';

/**
 * Farm plugin create command, create a rust farm plugin
 */
export function create(args: CreateArgs): void {
  console.log('call create', args, TEMPLATES_DIR);
  const dest = path.join(process.cwd(), args.dir);

  walk(TEMPLATES_DIR, (p, stat) => {
    if (stat.isFile()) {
      const content = readFileSync(p).toString();
    }
  });
}
