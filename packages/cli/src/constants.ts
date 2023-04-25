import { readFileSync } from 'node:fs';

const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);
export const VERSION = version as string;

export const DEFAULT_DEV_PORT = 9000;
