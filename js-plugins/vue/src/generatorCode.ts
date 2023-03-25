import path from "path";
import {
  compileScript,
  compileTemplate,
  compileStyle,
  SFCDescriptor,
  SFCScriptBlock,
  SFCTemplateBlock,
  BindingMetadata,
  rewriteDefault,
  SFCStyleBlock,
  SFCTemplateCompileResults,
} from "@vue/compiler-sfc";
import { error, warn, getHash, parsePath } from "./utils.js";
import { QueryObj, StylesCodeCache } from "./farm-vue-types.js";
import { cacheScript } from "./farm-vue-hmr.js";

const assignFilenameCode = genFileNameCode("App.vue");
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
  descriptor: SFCDescriptor,
  template: SFCTemplateBlock | null,
  filename: string,
  bindings: BindingMetadata,
  hasScoped: boolean,
  hash: string
): { code: string; map?: any } & Partial<SFCTemplateCompileResults> {
  if (template) {
    const result = compileTemplate({
      source: template.content,
      filename,
      id: filename,
      compilerOptions: {
        bindingMetadata: bindings ? bindings : undefined,
        scopeId: hasScoped ? `data-v-${hash}` : undefined,
      },
      inMap: template.map,
      slotted: descriptor.slotted,
      preprocessLang: template.lang,
      scoped: hasScoped,
    });
    const { code, map, errors, tips } = result;

    if (errors.length) {
      errors.forEach((err) => error({ id: filename, message: err }));
    }
    if (tips.length) {
      tips.forEach((tip) =>
        warn({
          id: filename,
          message: tip,
        })
      );
    }
    return {
      ...result,
      code: code.replace(/\nexport (function|const)/, "\n$1"),
    };
  }
  
  throw new Error("template is null");
}

export function genScriptCode(descriptor: SFCDescriptor, filename: string): {
  moduleType: string;
  code: string;
} & Partial<SFCScriptBlock> {
  let moduleType = "js";
  let code = "";
  let result: Partial<SFCScriptBlock> = {};
  const script = descriptor.script || descriptor.scriptSetup;
  // if script exist,add transformed code
  if (script) {
    const { content } = (result = compileScript(descriptor, {
      id: filename,
    }));
    cacheScript.set(descriptor, result);
    code += rewriteDefault(content, "_sfc_main");
    if (script && script.lang === "ts") moduleType = "ts";
  }
  // default script code
  else {
    code += defaultScriptCode;
  }
  return {
    moduleType,
    code,
    ...result,
  };
}

function genStyleCode(
  style: SFCStyleBlock,
  stylesCodeCache: StylesCodeCache,
  stylesCodeArr: string[],
  filename: string,
  hash: string,
  resolvedPath: string,
  index: number,
  isHmr: boolean = false
) {
  const {
    attrs: { lang = "css", scoped },
  } = style;
  const { code: styleCode, errors } = compileStyle({
    source: style.content,
    id: `data-v-${hash}`,
    scoped: Boolean(scoped),
    filename,
  });
  if (errors.length) {
    errors.forEach((err) => {
      error({ id: err.name, message: err.message });
    });
    return;
  }
  const queryStr = genQueryStr({
    lang,
    scoped: scoped ? hash : scoped,
    index,
    vue: true,
    t: isHmr ? Date.now() : 0,
  });

  const importPath = path.normalize(resolvedPath) + "?" + queryStr;

  const hashName = getHash(importPath);
  if (!stylesCodeCache[hashName]) {
    stylesCodeCache[hashName] = styleCode;
  }
  stylesCodeArr.push(
    "import " + JSON.stringify(importPath + `&hash=${hashName}`)
  );
}

export function genStylesCode(
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string,
  hash: string,
  filename: string,
  isHmr: boolean = false,
  deleteStyles: SFCStyleBlock[] = [],
  addStyles: SFCStyleBlock[] = []
) {
  const stylesCodeArr: string[] = [];
  const { styles } = descriptor;
  if (styles.length) {
    for (let i = 0; i < styles.length; i++) {
      genStyleCode(
        styles[i],
        stylesCodeCache,
        stylesCodeArr,
        filename,
        hash,
        resolvedPath,
        i,
        false
      );
    }
  }

  if (isHmr && addStyles.length) {
    for (let i = 0; i < addStyles.length; i++) {
      genStyleCode(
        styles[i],
        stylesCodeCache,
        stylesCodeArr,
        filename,
        hash,
        resolvedPath,
        i,
        true
      );
    }
  }
  return stylesCodeArr.join("\r\n");
}

export function genQueryStr(queryObj: QueryObj) {
  const queryStrArr: string[] = [];
  for (let key in queryObj) {
    if (queryObj[key] === 0 || queryObj[key])
      queryStrArr.push(`${key}=${queryObj[key]}`);
  }
  return queryStrArr.join("&");
}

export function genAssignHmrIdCode(hash: string) {
  return `_sfc_main.__hmrId = "${hash}"`;
}

export function genOtherCode(
  hasScoped: boolean,
  hash: string,
  isHmr = false,
  rerenderOnly: boolean
) {
  const otherCodeArr = [
    assignRenderCode,
    assignFilenameCode,
    hasScoped ? genAssignScopedCode(hash) : "",
    genAssignHmrIdCode(hash),
    defaultHmrCode,
    isHmr ? `_sfc_main._rerender_only=${rerenderOnly}` : "",
    exportDefaultCode,
  ];

  return otherCodeArr.join("\r\n");
}

export function genAssignScopedCode(hash: string) {
  return `_sfc_main.__scopeId = "data-v-${hash}";`;
}

export function genMainCode(
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string,
  isHmr: boolean = false,
  rerenderOnly: boolean = false,
  deleteStyles: SFCStyleBlock[] = [],
  addStyles: SFCStyleBlock[] = []
) {
  const output: string[] = [];
  const { template, scriptSetup, script, styles } = descriptor;
  const hasScoped = styles.some((style) => style.scoped);
  const hash = getHash(resolvedPath);
  const { filename } = parsePath(resolvedPath);

  const {
    code: scriptCode,
    map: scriptMap,
    moduleType,
    bindings,
  } = genScriptCode(descriptor, filename);

  const { code: templateCode, map: templateMap } = genTemplateCode(
    descriptor,
    template,
    filename,
    bindings || {},
    hasScoped,
    hash
  );
  const stylesCode = genStylesCode(
    descriptor,
    stylesCodeCache,
    resolvedPath,
    hash,
    filename,
    isHmr,
    deleteStyles,
    addStyles
  );
  const otherCode = genOtherCode(hasScoped, hash, isHmr, rerenderOnly);

  output.push(templateCode, scriptCode, stylesCode, otherCode);
  return {
    source: output.join("\r\n"),
    moduleType,
    map: "",
  };
}

export function genFileNameCode(resolvedPath: string) {
  return `_sfc_main.__file = "${resolvedPath}"`;
}
