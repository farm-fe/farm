/**
 * Make sure Farm's url rebase is the same as Vite. Following code is copied from Vite and modified by brightwu(吴明亮)
 * 
 * MIT License

Copyright (c) 2019-present, Yuxi (Evan) You and Vite contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

import path from 'node:path';
import fse from 'fs-extra';
import { normalizePath } from './share.js';

const nonEscapedDoubleQuoteRe = /(?<!\\)(")/g;

type CssUrlReplacer = (
  url: string,
  importer?: string
) => string | Promise<string>;
// https://drafts.csswg.org/css-syntax-3/#identifier-code-point
export const cssUrlRE =
  /(?<=^|[^\w\-\u0080-\uffff])url\((\s*('[^']+'|"[^"]+")\s*|[^'")]+)\)/;
export const cssDataUriRE =
  /(?<=^|[^\w\-\u0080-\uffff])data-uri\((\s*('[^']+'|"[^"]+")\s*|[^'")]+)\)/;
export const importCssRE = /@import ('[^']+\.css'|"[^"]+\.css"|[^'")]+\.css)/;
const functionCallRE = /^[A-Z_][\w-]*\(/i;

type ResolveFn = (id: string, importer?: string) => Promise<string | undefined>;

/**
 * relative url() inside \@imported sass and less files must be rebased to use
 * root file as base.
 */
export async function rebaseUrls(
  file: string,
  rootFile: string,
  variablePrefix: string,
  resolver: ResolveFn
): Promise<{
  file: string;
  contents?: string;
}> {
  file = path.resolve(file); // ensure os-specific flashes
  // in the same dir, no need to rebase
  const fileDir = path.dirname(file);
  const rootDir = path.dirname(rootFile);
  if (fileDir === rootDir) {
    return { file };
  }

  const content = await fse.readFile(file, 'utf-8');
  // no url()
  const hasUrls = cssUrlRE.test(content);
  // data-uri() calls
  const hasDataUris = cssDataUriRE.test(content);
  // no @import xxx.css
  const hasImportCss = importCssRE.test(content);

  if (!hasUrls && !hasDataUris && !hasImportCss) {
    return { file };
  }

  let rebased;
  const rebaseFn = async (url: string) => {
    if (url[0] === '/') return url;
    // ignore url's starting with variable
    if (url.startsWith(variablePrefix)) return url;

    const absolute = (await resolver(url, file)) || path.resolve(fileDir, url);
    const relative = path.relative(rootDir, absolute);
    return normalizePath(relative);
  };

  // fix css imports in less such as `@import "foo.css"`
  if (hasImportCss) {
    rebased = await rewriteImportCss(content, rebaseFn);
  }

  if (hasUrls) {
    rebased = await rewriteCssUrls(rebased || content, rebaseFn);
  }

  if (hasDataUris) {
    rebased = await rewriteCssDataUris(rebased || content, rebaseFn);
  }

  return {
    file,
    contents: rebased
  };
}

function rewriteImportCss(
  css: string,
  replacer: CssUrlReplacer
): Promise<string> {
  return asyncReplace(css, importCssRE, async (match) => {
    const [matched, rawUrl] = match;
    return await doImportCSSReplace(rawUrl, matched, replacer);
  });
}

function rewriteCssUrls(
  css: string,
  replacer: CssUrlReplacer
): Promise<string> {
  return asyncReplace(css, cssUrlRE, async (match) => {
    const [matched, rawUrl] = match;
    return await doUrlReplace(rawUrl.trim(), matched, replacer);
  });
}

function rewriteCssDataUris(
  css: string,
  replacer: CssUrlReplacer
): Promise<string> {
  return asyncReplace(css, cssDataUriRE, async (match) => {
    const [matched, rawUrl] = match;
    return await doUrlReplace(rawUrl.trim(), matched, replacer, 'data-uri');
  });
}

function skipUrlReplacer(rawUrl: string) {
  return (
    isExternalUrl(rawUrl) ||
    isDataUrl(rawUrl) ||
    rawUrl[0] === '#' ||
    functionCallRE.test(rawUrl)
  );
}
async function doUrlReplace(
  rawUrl: string,
  matched: string,
  replacer: CssUrlReplacer,
  funcName = 'url'
) {
  let wrap = '';
  const first = rawUrl[0];
  if (first === `"` || first === `'`) {
    wrap = first;
    rawUrl = rawUrl.slice(1, -1);
  }

  if (skipUrlReplacer(rawUrl)) {
    return matched;
  }

  let newUrl = await replacer(rawUrl);
  // The new url might need wrapping even if the original did not have it, e.g. if a space was added during replacement
  if (wrap === '' && newUrl !== encodeURI(newUrl)) {
    wrap = '"';
  }
  // If wrapping in single quotes and newUrl also contains single quotes, switch to double quotes.
  // Give preference to double quotes since SVG inlining converts double quotes to single quotes.
  if (wrap === "'" && newUrl.includes("'")) {
    wrap = '"';
  }
  // Escape double quotes if they exist (they also tend to be rarer than single quotes)
  if (wrap === '"' && newUrl.includes('"')) {
    newUrl = newUrl.replace(nonEscapedDoubleQuoteRe, '\\"');
  }
  return `${funcName}(${wrap}${newUrl}${wrap})`;
}

async function doImportCSSReplace(
  rawUrl: string,
  matched: string,
  replacer: CssUrlReplacer
) {
  let wrap = '';
  const first = rawUrl[0];
  if (first === `"` || first === `'`) {
    wrap = first;
    rawUrl = rawUrl.slice(1, -1);
  }
  if (isExternalUrl(rawUrl) || isDataUrl(rawUrl) || rawUrl[0] === '#') {
    return matched;
  }

  return `@import ${wrap}${await replacer(rawUrl)}${wrap}`;
}

async function asyncReplace(
  input: string,
  re: RegExp,
  replacer: (match: RegExpExecArray) => string | Promise<string>
): Promise<string> {
  let match: RegExpExecArray | null;
  let remaining = input;
  let rewritten = '';
  while ((match = re.exec(remaining))) {
    rewritten += remaining.slice(0, match.index);
    rewritten += await replacer(match);
    remaining = remaining.slice(match.index + match[0].length);
  }
  rewritten += remaining;
  return rewritten;
}

export const externalRE = /^(https?:)?\/\//;
export const isExternalUrl = (url: string): boolean => externalRE.test(url);

export const dataUrlRE = /^\s*data:/i;
export const isDataUrl = (url: string): boolean => dataUrlRE.test(url);
