export const pluginName = 'farm-less-plugin';

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}
