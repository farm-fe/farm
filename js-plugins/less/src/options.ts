import pkg from '../package.json';

export const pluginName = pkg.name;

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}
