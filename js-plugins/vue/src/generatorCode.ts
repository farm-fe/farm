import path from 'path';
import {
  EncodedSourceMap,
  addMapping,
  fromMap,
  toEncodedMap
} from '@jridgewell/gen-mapping';
import { TraceMap, eachMapping } from '@jridgewell/trace-mapping';
import {
  BindingMetadata,
  CompilerOptions,
  SFCDescriptor,
  SFCScriptBlock,
  SFCScriptCompileOptions,
  SFCStyleBlock,
  SFCTemplateBlock,
  SFCTemplateCompileResults,
  compileScript,
  compileTemplate,
  rewriteDefault
} from '@vue/compiler-sfc';
import { RawSourceMap } from 'source-map';
import { cacheScript, isEqualBlock } from './farm-vue-hmr.js';
import {
  QueryObj,
  ResolvedOptions,
  StylesCodeCache,
  Union
} from './farm-vue-types.js';
import { error, getHash, parsePath, warn } from './utils.js';

type SourceMap = Omit<RawSourceMap, 'version'> & { version: 3 };

const assignFilenameCode = genFileNameCode('App.vue');
const assignRenderCode = `_sfc_main.render = typeof render === "function" ? render : undefined`;
const exportDefaultCode = `export default _sfc_main`;
const defaultScriptCode = `const _sfc_main = {}`;
const defaultHmrCode = `
typeof __VUE_HMR_RUNTIME__ !== "undefined" && __VUE_HMR_RUNTIME__.createRecord(_sfc_main.__hmrId, _sfc_main);
module.meta.hot.accept((mod) => {
  if (!mod)
    return;
  const { default: updated } = mod;
  if (updated._rerender_only) {
    __VUE_HMR_RUNTIME__.rerender(updated.__hmrId, updated.render);
  } else {
    __VUE_HMR_RUNTIME__.reload(updated.__hmrId, updated);
  }
});
`;

export function genTemplateCode(
  templateCompilerOptions: ResolvedOptions['template'],
  descriptor: SFCDescriptor,
  template: SFCTemplateBlock | null,
  filename: string,
  bindings: BindingMetadata,
  hasScoped: boolean,
  hash: string
):
  | Union<SFCTemplateCompileResults, { code: string }>
  | { code: string; map: RawSourceMap } {
  if (template) {
    // if using TS, support TS syntax in template expressions
    const expressionPlugins: CompilerOptions['expressionPlugins'] =
      templateCompilerOptions?.compilerOptions?.expressionPlugins ?? [];
    const lang = descriptor.scriptSetup?.lang || descriptor.script?.lang;
    if (
      lang &&
      /tsx?$/.test(lang) &&
      !expressionPlugins.includes('typescript')
    ) {
      expressionPlugins.push('typescript');
    }

    const result = compileTemplate({
      source: template.content,
      filename,
      id: hash,
      compilerOptions: {
        ...templateCompilerOptions?.compilerOptions,
        bindingMetadata: bindings ? bindings : undefined,
        scopeId: hasScoped ? `data-v-${hash}` : undefined,
        expressionPlugins
      },
      inMap: template.map,
      slotted: descriptor.slotted,
      preprocessLang: template.lang,
      scoped: hasScoped,
      ...templateCompilerOptions
    });
    const { code, map, errors, tips } = result;

    if (errors.length) {
      errors.forEach((err) => error({ id: filename, message: err }));
    }
    if (tips.length) {
      tips.forEach((tip) =>
        warn({
          id: filename,
          message: tip
        })
      );
    }
    return {
      ...result,
      code: code.replace(/\nexport (function|const)/, '\n$1')
    };
  }
  return {
    code: '',
    map: {} as RawSourceMap
  };
}

export function genScriptCode(
  scriptCompilerOptions: ResolvedOptions['script'],
  descriptor: SFCDescriptor,
  hash: string
): Union<SFCScriptBlock, { code: string; moduleType: string }> {
  let moduleType = 'js';
  let code = '';
  let result: SFCScriptBlock = {} as SFCScriptBlock;
  const script = descriptor.script || descriptor.scriptSetup;
  const babelParserPlugins: SFCScriptCompileOptions['babelParserPlugins'] = [];

  if (script?.lang === 'ts' || script?.lang === 'tsx') {
    babelParserPlugins.push('typescript');
  }
  if (script?.lang === 'jsx' || script?.lang === 'tsx') {
    babelParserPlugins.push('jsx');
  }

  if (scriptCompilerOptions.babelParserPlugins) {
    babelParserPlugins.push(...scriptCompilerOptions.babelParserPlugins);
  }

  // if script exist, add transformed code
  if (script) {
    const { content } = (result = compileScript(descriptor, {
      id: hash,
      ...scriptCompilerOptions,
      babelParserPlugins
    }));
    cacheScript.set(descriptor, result);
    code += rewriteDefault(content, '_sfc_main', babelParserPlugins);
    if (script?.lang) moduleType = script.lang;
  }
  // default script code
  else {
    code += defaultScriptCode;
  }
  return {
    moduleType,
    code,
    ...result
  };
}

function genStyleCode(
  styleCompilerOptions: ResolvedOptions['style'],
  style: SFCStyleBlock,
  stylesCodeCache: StylesCodeCache,
  stylesCodeArr: string[],
  filename: string,
  hash: string,
  resolvedPath: string,
  index: number,
  isHmr = false
) {
  const {
    attrs: { lang = 'css', scoped }
  } = style;

  const queryStr = genQueryStr({
    lang,
    scoped: scoped ? hash : scoped,
    index,
    vue: true,
    t: isHmr ? Date.now() : 0
  });

  const importPath = path.normalize(resolvedPath) + '?' + queryStr;

  const hashName = getHash(importPath);
  if (!stylesCodeCache[hashName]) {
    stylesCodeCache[hashName] = style.content;
  }

  stylesCodeArr.push(
    'import ' + JSON.stringify(importPath + `&hash=${hashName}`)
  );
}

export function genStylesCode(
  styleCompilerOptions: ResolvedOptions['style'],
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string,
  hash: string,
  filename: string,
  isHmr = false,
  deleteStyles: SFCStyleBlock[] = [],
  addStyles: SFCStyleBlock[] = []
) {
  const stylesCodeArr: string[] = [];
  const { styles } = descriptor;
  if (styles.length) {
    for (let i = 0; i < styles.length; i++) {
      // // if style is deleted, skip
      // if (deleteStyles.some((ds) => isEqualBlock(ds, styles[i]))) continue;

      genStyleCode(
        styleCompilerOptions,
        styles[i],
        stylesCodeCache,
        stylesCodeArr,
        filename,
        hash,
        resolvedPath,
        i,
        isHmr
      );
    }
  }

  if (isHmr && addStyles.length) {
    for (let i = 0; i < addStyles.length; i++) {
      if (styles.some((ds) => isEqualBlock(ds, addStyles[i]))) continue;

      genStyleCode(
        styleCompilerOptions,
        styles[i],
        stylesCodeCache,
        stylesCodeArr,
        filename,
        hash,
        resolvedPath,
        i,
        isHmr
      );
    }
  }

  return stylesCodeArr.join('\r\n');
}

export function genQueryStr(queryObj: QueryObj) {
  const queryStrArr: string[] = [];
  for (const key in queryObj) {
    if (queryObj[key] === 0 || queryObj[key])
      queryStrArr.push(`${key}=${queryObj[key]}`);
  }
  return queryStrArr.join('&');
}

export function genAssignHmrIdCode(hash: string) {
  return `_sfc_main.__hmrId = "${hash}"`;
}

export function genOtherCode(
  hasScoped: boolean,
  hash: string,
  isHmr = false,
  rerenderOnly: boolean,
  filename: string
) {
  const otherCodeArr = [
    assignRenderCode,
    genFileNameCode(filename),
    hasScoped ? genAssignScopedCode(hash) : '',
    genAssignHmrIdCode(hash),
    isHmr ? defaultHmrCode : '',
    isHmr ? `_sfc_main._rerender_only=${rerenderOnly}` : '',
    exportDefaultCode
  ];

  return otherCodeArr.join('\r\n');
}

export function genAssignScopedCode(hash: string) {
  return `_sfc_main.__scopeId = "data-v-${hash}";`;
}

export function genMainCode(
  resolvedOptions: ResolvedOptions,
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string,
  isHmr = false,
  rerenderOnly = false,
  deleteStyles: SFCStyleBlock[] = [],
  addStyles: SFCStyleBlock[] = []
) {
  const {
    template: templateCompilerOptions,
    script: scriptCompilerOptions,
    style: styleCompilerOptions,
    sourceMap
  } = resolvedOptions;
  const output: string[] = [];
  const { template, scriptSetup, script, styles } = descriptor;
  const hasScoped = styles.some((style) => style.scoped);
  const hash = getHash(resolvedPath);
  const { filePath } = parsePath(resolvedPath);

  const {
    code: scriptCode,
    map: scriptMap,
    moduleType,
    bindings
  } = genScriptCode(scriptCompilerOptions, descriptor, hash);

  const { code: templateCode, map: templateMap } = genTemplateCode(
    templateCompilerOptions,
    descriptor,
    template,
    filePath,
    bindings || {},
    hasScoped,
    hash
  );
  let resolvedMap: EncodedSourceMap | string = '';
  //only "sourceMap === true" should generator source-map
  if ((templateMap || scriptMap) && sourceMap) {
    resolvedMap = genSourceMap(
      scriptMap as unknown as SourceMap,
      templateMap as unknown as SourceMap,
      templateCode
    );
  }

  const stylesCode = genStylesCode(
    styleCompilerOptions,
    descriptor,
    stylesCodeCache,
    resolvedPath,
    hash,
    filePath,
    isHmr,
    deleteStyles,
    addStyles
  );
  const otherCode = genOtherCode(
    hasScoped,
    hash,
    isHmr,
    rerenderOnly,
    filePath
  );

  output.push(scriptCode, templateCode, stylesCode, otherCode);
  return {
    source: output.join('\r\n'),
    moduleType,
    map:
      typeof resolvedMap === 'string'
        ? resolvedMap
        : JSON.stringify(resolvedMap)
  };
}

function genSourceMap(
  scriptMap: SourceMap,
  templateMap: SourceMap,
  scriptCode: string
) {
  //gen sourceMap
  let resolvedMap: EncodedSourceMap | undefined = void 0;
  if (scriptMap && templateMap) {
    const gen = fromMap(scriptMap);
    const tracer = new TraceMap(templateMap);
    const offset = (scriptCode.match(/\r?\n/g)?.length ?? 0) + 1;
    eachMapping(tracer, (m) => {
      if (m.source == null) return;
      addMapping(gen, {
        source: m.source,
        original: { line: m.originalLine, column: m.originalColumn },
        generated: {
          line: m.generatedLine + offset,
          column: m.generatedColumn
        }
      });
    });
    resolvedMap = toEncodedMap(gen);
    resolvedMap.sourcesContent = templateMap.sourcesContent;
  } else {
    resolvedMap = scriptMap ?? templateMap;
  }
  return resolvedMap;
}

export function genFileNameCode(resolvedPath: string) {
  return `_sfc_main.__file = "${resolvedPath}"`;
}
