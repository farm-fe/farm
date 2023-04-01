import { isAbsolute, join } from 'path';
import type { SassOptions } from '.';
import fs from 'fs';

export const getAdditionContext = (cwd: string, option: SassOptions) => {
  const { globals = [], content } = option;

  const result = globals.reduce((result, file) => {
    let filepath: string;
    if (isAbsolute(file)) {
      filepath = file;
    } else {
      filepath = join(cwd, file);
    }

    result.push(fs.readFileSync(filepath, 'utf-8'));
    return result;
  }, []);

  if (content) {
    result.push(content);
  }

  return result.join('\n');
};
