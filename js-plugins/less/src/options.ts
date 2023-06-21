export const pluginName = '@farmfe/js-plugin-less';

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}
