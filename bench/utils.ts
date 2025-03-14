import path from 'node:path';

export const cwd = process.cwd();

export const getExampleRoot = (name: string) => {
  return path.join(cwd, '../examples', name);
};
