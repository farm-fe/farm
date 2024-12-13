const postfixRE = /[?#].*$/;
export function cleanUrl(url: string): string {
  return url.replace(postfixRE, '');
}

const importQueryRE = /(\?|&)import=?(?:&|$)/;
export const isImportRequest = (url: string): boolean =>
  importQueryRE.test(url);

const trailingSeparatorRE = /[?&]$/;
export function removeImportQuery(url: string): string {
  return url.replace(importQueryRE, '$1').replace(trailingSeparatorRE, '');
}

export const knownJavascriptExtensionRE = /\.[tj]sx?$/;

export const urlRE = /(\?|&)url(?:&|$)/;
