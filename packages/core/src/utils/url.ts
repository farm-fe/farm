const postfixRE = /[?#].*$/;
export function cleanUrl(url: string): string {
  return url.replace(postfixRE, '');
}

const importQueryRE = /(\?|&)import=?(?:&|$)/;
export const isImportRequest = (url: string): boolean =>
  importQueryRE.test(url);

export const knownJavascriptExtensionRE = /\.[tj]sx?$/;
