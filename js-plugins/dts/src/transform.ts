import { dirname, relative, isAbsolute } from 'node:path';
import { isRegExp, normalizePath } from './utils.js';

const globSuffixRE = /^((?:.*\.[^.]+)|(?:\*+))$/;
const globalDynamicTypeRE = /import\(['"][^;\n]+?['"]\)\.\w+[.()[\]<>,;\n\s]/g;
const dynamicTypeRE = /import\(['"](.+)['"]\)\.(.+)([.()[\]<>,;\n\s])/;
const importTypesRE = /import\s?(?:type)?\s?\{(.+)\}\s?from\s?['"].+['"]/;

export function normalizeGlob(path: string) {
  if (/[\\/]$/.test(path)) {
    return path + '**';
  } else if (!globSuffixRE.test(path.split(/[\\/]/).pop()!)) {
    return path + '/**';
  }

  return path;
}

export function transformDynamicImport(content: string) {
  const importMap = new Map<string, Set<string>>();

  content = content.replace(globalDynamicTypeRE, (str) => {
    const matchResult = str.match(dynamicTypeRE)!;
    const libName = matchResult[1];
    const importSet =
      importMap.get(libName) ??
      importMap.set(libName, new Set<string>()).get(libName)!;
    const usedType = matchResult[2];

    importSet.add(usedType);

    return usedType + matchResult[3];
  });

  importMap.forEach((importSet, libName) => {
    const importReg = new RegExp(
      `import\\s?(?:type)?\\s?\\{[^;\\n]+\\}\\s?from\\s?['"]${libName}['"]`,
      'g'
    );
    const matchResult = content.match(importReg);

    if (matchResult?.[0]) {
      const importedTypes = matchResult[0]
        .match(importTypesRE)![1]
        .trim()
        .split(',');

      content = content.replace(
        matchResult[0],
        `import type { ${Array.from(importSet)
          .concat(importedTypes)
          .join(', ')} } from '${libName}'`
      );
    } else {
      content =
        `import type { ${Array.from(importSet).join(
          ', '
        )} } from '${libName}';\n` + content;
    }
  });

  return content;
}

const globalImportRE =
  /(?:(?:import|export)\s?(?:type)?\s?(?:(?:\{[^;\n]+\})|(?:[^;\n]+))\s?from\s?['"][^;\n]+['"])|(?:import\(['"][^;\n]+?['"]\))/g;
const staticImportRE =
  /(?:import|export)\s?(?:type)?\s?\{?.+\}?\s?from\s?['"](.+)['"]/;
const dynamicImportRE = /import\(['"]([^;\n]+?)['"]\)/;
const simpleStaticImportRE = /((?:import|export).+from\s?)['"](.+)['"]/;
const simpleDynamicImportRE = /(import\()['"](.+)['"]\)/;

function isAliasMatch(alias: any, importee: string) {
  if (isRegExp(alias.find)) return alias.find.test(importee);
  if (importee.length < alias.find.length) return false;
  if (importee === alias.find) return true;

  return (
    importee.indexOf(alias.find) === 0 &&
    (alias.find.endsWith('/') ||
      importee.substring(alias.find.length)[0] === '/')
  );
}

export function transformAliasImport(
  filePath: string,
  content: string,
  aliases: any[],
  exclude: (string | RegExp)[] = []
) {
  if (!aliases.length) return content;

  return content.replace(globalImportRE, (str) => {
    let matchResult = str.match(staticImportRE);
    let isDynamic = false;

    if (!matchResult) {
      matchResult = str.match(dynamicImportRE);
      isDynamic = true;
    }

    if (matchResult?.[1]) {
      const matchedAlias = aliases.find((alias) =>
        isAliasMatch(alias, matchResult![1])
      );

      if (matchedAlias) {
        if (
          exclude.some((e) =>
            isRegExp(e)
              ? e.test(matchResult![1])
              : String(e) === matchResult![1]
          )
        ) {
          return str;
        }

        const truthPath = isAbsolute(matchedAlias.replacement)
          ? normalizePath(relative(dirname(filePath), matchedAlias.replacement))
          : normalizePath(matchedAlias.replacement);

        return str.replace(
          isDynamic ? simpleDynamicImportRE : simpleStaticImportRE,
          `$1'${matchResult[1].replace(
            matchedAlias.find,
            (truthPath.startsWith('.') ? truthPath : `./${truthPath}`) +
              (typeof matchedAlias.find === 'string' &&
              matchedAlias.find.endsWith('/')
                ? '/'
                : '')
          )}'${isDynamic ? ')' : ''}`
        );
      }
    }

    return str;
  });
}

const pureImportRE = /import\s?['"][^;\n]+?['"];?\n?/g;

export function removePureImport(content: string) {
  return content.replace(pureImportRE, '');
}

const setupFunctionRE =
  /function setup\([\s\S]+\)\s+?\{[\s\S]+return __returned__\n\}/;

export function transferSetupPosition(content: string) {
  const match = content.match(setupFunctionRE);

  if (match) {
    const setupFunction = match[0];

    return content
      .replace(setupFunction, '')
      .replace('setup})', setupFunction.slice('function '.length) + '\n\r})');
  }

  return content;
}
