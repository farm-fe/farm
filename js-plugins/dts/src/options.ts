export const pluginName = "farm-plugin-dts";

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}
