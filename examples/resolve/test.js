// import '@generated-content-virtual-file'
function text(selector, text) {
  document.querySelector(selector).textContent = text;
}
// import set from '@antv/util/lib/set';
// console.log(set);
// import * as constants from 'focus-lock/constants'
// console.log(constants)
// import * as ele from 'electron-to-chromium/versions'
// console.log(ele)
// import from a utf-8 bom file
// import { msg as bomMsg } from './utf8-bom/main.js';
import bomMsg from './rel';
text('.utf8-bom', bomMsg);

// deep import
// import slicedToArray from '@babel/runtime/helpers/esm/slicedToArray'

// const iterable = (function* () {
//   yield 2
//   yield 4
//   yield 6
//   yield 8
// })()

// text('.deep-import', JSON.stringify(slicedToArray(iterable, 2)))

// import exportsAndNestedScopeMsg from '@vitejs/test-resolve-exports-and-nested-scope/nested'
// text('.exports-and-nested-scope', exportsAndNestedScopeMsg)

// // exports field
// import { msg } from '@vitejs/test-resolve-exports-path'
// text('.exports-entry', msg)

// // deep import w/ exports
// import { msg as deepMsg } from '@vitejs/test-resolve-exports-path/deep.js'
// text('.exports-deep', deepMsg)

// // deep import w/ exports w/ query
// import deepPath from '@vitejs/test-resolve-exports-path/deep.json?url'
// text('.exports-deep-query', deepPath)

// // deep import w/ exposed dir
// import { msg as exposedDirMsg } from '@vitejs/test-resolve-exports-path/dir/dir'
// text('.exports-deep-exposed-dir', exposedDirMsg)

// // deep import w/ mapped dir
// import { msg as mappedDirMsg } from '@vitejs/test-resolve-exports-path/dir-mapped/dir'
// text('.exports-deep-mapped-dir', mappedDirMsg)

// import { msg as exportsEnvMsg } from '@vitejs/test-resolve-exports-env'
// text('.exports-env', exportsEnvMsg)

// import { msg as exportsFromRootMsg } from '@vitejs/test-resolve-exports-from-root/nested'
// text('.exports-from-root', exportsFromRootMsg)

// import { msg as exportsLegacyFallbackMsg } from '@vitejs/test-resolve-exports-legacy-fallback/dir'
// text('.exports-legacy-fallback', exportsLegacyFallbackMsg)

// import { msg as exportsWithModule } from '@vitejs/test-resolve-exports-with-module'
// text('.exports-with-module', exportsWithModule)

// import { msg as exportsWithModuleCondition } from '@vitejs/test-resolve-exports-with-module-condition'
// import { msg as exportsWithModuleConditionRequired } from '@vitejs/test-resolve-exports-with-module-condition-required'
// text('.exports-with-module-condition', exportsWithModuleCondition)
// text(
//   '.exports-with-module-condition-required',
//   exportsWithModuleConditionRequired,
// )

// // imports field
// import { msg as importsTopLevel } from '#top-level'
// text('.imports-top-level', importsTopLevel)

// import { msg as importsSameLevel } from '#same-level'
// text('.imports-same-level', importsSameLevel)

// import { msg as importsNested } from '#nested/path.js'
// text('.imports-nested', importsNested)

// import { msg as importsStar } from '#star/index.js'
// text('.imports-star', importsStar)

// import { msg as importsSlash } from '#slash/index.js'
// text('.imports-slash', importsSlash)

// import { msg as importsPkgSlash } from '#other-pkg-slash/index.js'
// text('.imports-pkg-slash', importsPkgSlash)

// // implicit index resolving
// import { foo } from './util'
// text('.index', foo())

// // implicit dir index vs. file
// import { file } from './dir'
// text('.dir-vs-file', file)

// // exact extension vs. duplicated (.js.js)
// import { file as exactExtMsg } from './exact-extension/file.js'
// text('.exact-extension', exactExtMsg)

// // nested extension
// import { file as fileJsonMsg } from './exact-extension/file.json'
// text('.nested-extension', fileJsonMsg)

// // don't add extensions to dir name (./dir-with-ext.js/index.js)
// import { file as dirWithExtMsg } from './dir-with-ext'
// text('.dir-with-ext', dirWithExtMsg)

// import { msg as tsExtensionMsg } from './ts-extension'
// text('.ts-extension', tsExtensionMsg)

// import { msgJsx as tsJsxExtensionMsg } from './ts-extension'
// text('.jsx-extension', tsJsxExtensionMsg)

// import { msgTsx as tsTsxExtensionMsg } from './ts-extension'
// text('.tsx-extension', tsTsxExtensionMsg)

// import { msgCjs as tsCjsExtensionMsg } from './ts-extension'
// text('.cjs-extension', tsCjsExtensionMsg)

// import { msgMjs as tsMjsExtensionMsg } from './ts-extension'
// text('.mjs-extension', tsMjsExtensionMsg)

// import { msgMjs as tsMjsExtensionWithQueryMsg } from './ts-extension?query=1'
// text('.mjs-extension-with-query', tsMjsExtensionWithQueryMsg)

// // filename with dot
// import { bar } from './util/bar.util'
// text('.dot', bar())

// // browser field
// import main from '@vitejs/test-resolve-browser-field/no-ext'
// text('.browser', main)
// import a from '@vitejs/test-resolve-browser-field/no-ext'
// import b from '@vitejs/test-resolve-browser-field/no-ext.js' // no substitution
// import c from '@vitejs/test-resolve-browser-field/ext'
// import d from '@vitejs/test-resolve-browser-field/ext.js'
// import e from '@vitejs/test-resolve-browser-field/ext-index/index.js'
// import f from '@vitejs/test-resolve-browser-field/ext-index'
// import g from '@vitejs/test-resolve-browser-field/no-ext-index/index.js' // no substitution
// import h from '@vitejs/test-resolve-browser-field/no-ext?query'
// import i from '@vitejs/test-resolve-browser-field/bare-import'

// import {
//   ra,
//   rb,
//   rc,
//   rd,
//   re,
//   rf,
//   rg,
// } from '@vitejs/test-resolve-browser-field/relative'

// const success = [main, a, c, d, e, f, h, i, ra, rc, rd, re, rf]
// const noSuccess = [b, g, rb, rg]

// if (
//   [...success, ...noSuccess].filter((text) => text.includes('[success]'))
//     .length === success.length
// ) {
//   text('.browser', main)
// }

// import browserModule1 from '@vitejs/test-resolve-browser-module-field1'
// text('.browser-module1', browserModule1)

// import browserModule2 from '@vitejs/test-resolve-browser-module-field2'
// text('.browser-module2', browserModule2)

// import browserModule3 from '@vitejs/test-resolve-browser-module-field3'
// text('.browser-module3', browserModule3)

// import { msg as requireButWithModuleFieldMsg } from '@vitejs/test-require-pkg-with-module-field'
// text('.require-pkg-with-module-field', requireButWithModuleFieldMsg)

// import { msg as customExtMsg } from './custom-ext'
// text('.custom-ext', customExtMsg)

// import { msg as customMainMsg } from '@vitejs/test-resolve-custom-main-field'
// text('.custom-main-fields', customMainMsg)

// import { msg as customConditionMsg } from '@vitejs/test-resolve-custom-condition'
// text('.custom-condition', customConditionMsg)

// // should be ok to import a file marked with browser: false
// import '@vitejs/test-resolve-browser-field/not-browser'
// import '@vitejs/test-resolve-browser-field/multiple.dot.path'

// // css entry
// import css from 'normalize.css?inline'
// if (typeof css === 'string') {
//   text('.css', '[success] resolve package with css entry file')
// }

// // monorepo linked dep w/ upper directory import
// import { msg as linkedMsg } from '@vitejs/test-resolve-linked'
// text('.monorepo', linkedMsg)

// import { msg as virtualMsg } from '@virtual-file'
// text('.virtual', virtualMsg)

// import { msg as virtualMsg9036 } from 'virtual:file-9036.js'
// text('.virtual-9036', virtualMsg9036)

// import { msg as customVirtualMsg } from '@custom-virtual-file'
// text('.custom-virtual', customVirtualMsg)

// import { msg as inlineMsg } from './inline-package'
// text('.inline-pkg', inlineMsg)

// import es5Ext from 'es5-ext'
// import contains from 'es5-ext/string/#/contains'
// import { last } from '@vitejs/test-resolve-sharp-dir'

// text(
//   '.path-contains-sharp-symbol',
//   `[success] ${contains.call('#', '#')} ${last.call('#')}`,
// )
