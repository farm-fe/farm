import path from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';
import MagicString from 'magic-string';

// Private browser-only feasibility prototype. No shared or static remote imports.
export function rewriteRemoteImports(content, filename, aliases) {
  const source = ts.createSourceFile(filename, content, ts.ScriptTarget.Latest, true,
    /\.tsx$/.test(filename) ? ts.ScriptKind.TSX : /\.jsx$/.test(filename) ? ts.ScriptKind.JSX : ts.ScriptKind.TS);
  const remote = value => aliases.some(alias => value.startsWith(`${alias}/`));
  let binding = '__farm_mf_load__';
  while (content.includes(binding)) binding += '_';
  const edits = [];
  function visit(node) {
    if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
        node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier) && remote(node.moduleSpecifier.text)) {
      throw new Error(`[Farm MF] Static remote imports/exports are not supported: ${node.moduleSpecifier.text}`);
    }
    if (ts.isCallExpression(node) && node.expression.kind === ts.SyntaxKind.ImportKeyword) {
      const arg = node.arguments[0];
      if (arg && (ts.isStringLiteral(arg) || ts.isNoSubstitutionTemplateLiteral(arg))) {
        if (remote(arg.text)) {
          if (node.arguments.length !== 1) throw new Error('[Farm MF] Remote import options are not supported');
          edits.push([node.getStart(source), node.end, `${binding}(${JSON.stringify(arg.text)})`]);
        }
      } else if (arg) {
        // Only reject expressions recognizably belonging to a configured remote;
        // arbitrary expressions cannot be classified safely and remain Farm imports.
        const prefix = ts.isTemplateExpression(arg) ? arg.head.text :
          ts.isBinaryExpression(arg) && ts.isStringLiteral(arg.left) ? arg.left.text : '';
        if (remote(prefix)) throw new Error('[Farm MF] Computed remote imports are not supported');
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  if (!edits.length) return null;
  const result = new MagicString(content);
  for (const [start, end, replacement] of edits) result.overwrite(start, end, replacement);
  result.prepend(`import { loadRemote as ${binding} } from 'farm-mf:runtime';\n`);
  return { content: result.toString(), sourceMap: result.generateMap({source:filename, includeContent:true, hires:true}).toString() };
}

export default function federation(options) {
  if (!options || !/^[A-Za-z_$][\w$]*$/.test(options.name ?? '')) throw new Error('[Farm MF] name must be a JavaScript identifier');
  if (options.shared !== undefined) throw new Error('[Farm MF] shared is not supported by this probe');
  const { name, exposes = {}, remotes = {}, filename = 'remoteEntry.js' } = options;
  if (!/^[\w.-]+\.js$/.test(filename)) throw new Error('[Farm MF] filename must be a plain .js filename');
  const namespace = `farm_mf_${name}`;
  let root;
  let containerPath;
  let runtimePath;
  let remotePublicPath;
  const remoteList = Object.entries(remotes).map(([alias, value]) => {
    const url = new URL(value.entry);
    if (!['http:', 'https:'].includes(url.protocol)) throw new Error('[Farm MF] Remote entry must be HTTP(S)');
    return {name:value.name, alias, entry:url.href, type:'module', entryGlobalName:value.name};
  });
  return {
    name: '@farmfe/js-plugin-module-federation-experiment', priority: 1100,
    config(config) {
      root = path.resolve(config.root ?? config.compilation?.root ?? process.cwd());
      containerPath = path.join(root, `.farm-mf-${name}-container.js`);
      runtimePath = path.join(root, `.farm-mf-${name}-runtime.js`);
      const compilation = config.compilation ?? {};
      if (compilation.runtime?.namespace && compilation.runtime.namespace !== namespace) throw new Error('[Farm MF] Conflicting runtime.namespace');
      if (compilation.output?.targetEnv && !compilation.output.targetEnv.startsWith('browser')) throw new Error('[Farm MF] Browser target required');
      if (Object.keys(exposes).length && compilation.input && Object.keys(compilation.input).length) throw new Error('[Farm MF] Probe remote must use a dedicated container build');
      if (Object.keys(exposes).length) {
        const publicPath = compilation.output?.publicPath;
        if (!publicPath) throw new Error('[Farm MF] Remote requires publicPath');
        remotePublicPath = new URL(publicPath, config.server?.origin).href;
        if (!/^https?:\/\//.test(remotePublicPath)) throw new Error('[Farm MF] Remote requires an HTTP(S) publicPath or server.origin');
        remotePublicPath = remotePublicPath.replace(/\/?$/, '/');
      }
      for (const key of Object.keys(exposes)) if (!key.startsWith('./')) throw new Error('[Farm MF] Expose keys must start with ./');
      return {...config, compilation:{...compilation,
        runtime:{...compilation.runtime, namespace, isolate:true},
        ...(Object.keys(exposes).length ? { input:{remoteEntry:'farm-mf:container'},
          output:{...compilation.output, entryFilename:filename} } : {})
      }};
    },
    resolve: {filters:{sources:['^(\\./)?farm-mf:'], importers:['.*']}, executor({source}) {
      source = source.replace(/^\.\//, '');
      const resolvedPath = source === 'farm-mf:container' ? containerPath : source === 'farm-mf:runtime' ? runtimePath : null;
      if (!resolvedPath) throw new Error(`[Farm MF] Unknown virtual module ${source}`);
      return {resolvedPath, query:[], sideEffects:true};
    }},
    load: {filters:{resolvedPaths:['\\.farm-mf-.*-(container|runtime)\\.js$']}, executor({resolvedPath}) {
      if (resolvedPath === containerPath) {
        const loaders = Object.entries(exposes).map(([key, value]) => `${JSON.stringify(key)}: () => import(${JSON.stringify(path.resolve(root, value))})`).join(',\n');
        return {moduleType:'js', content:`
          globalThis[${JSON.stringify(namespace)}].m.sp([${JSON.stringify(remotePublicPath)}]);
          const loaders = {${loaders}};
          let initialized = false, scope;
          const container = {
            async init(nextScope) {
              if (nextScope && Object.keys(nextScope).length) throw new Error('[Farm MF ${name}] shared is not supported');
              if (initialized && scope !== nextScope) throw new Error('[Farm MF ${name}] Different share scope');
              scope = nextScope; initialized = true;
            },
            async get(key) {
              if (!initialized) throw new Error('[Farm MF ${name}] init must run before get');
              if (!Object.prototype.hasOwnProperty.call(loaders, key)) throw new Error('[Farm MF ${name}] Unknown expose: ' + key);
              const module = await loaders[key]();
              return () => module;
            }
          };
          if (globalThis[${JSON.stringify(name)}]) throw new Error('[Farm MF ${name}] Duplicate container name');
          globalThis[${JSON.stringify(name)}] = container;
          export const get = container.get;
          export const init = container.init;
        `};
      }
      if (resolvedPath === runtimePath) {
        const runtimeEntry = fileURLToPath(import.meta.resolve('@module-federation/runtime'));
        return {moduleType:'js', content:`
          import {createInstance} from ${JSON.stringify(runtimeEntry)};
          const remotes = ${JSON.stringify(remoteList)};
          const instance = createInstance({name:${JSON.stringify(name)}, remotes});
          export async function loadRemote(id) {
            try {
              const result = await instance.loadRemote(id);
              if (result == null) throw new Error('Runtime returned no module');
              return result;
            } catch (cause) {
              const remote = remotes.find(r => id.startsWith(r.alias + '/'));
              throw new Error('[Farm MF ${name}] Failed ' + id + ' from ' + remote?.entry + ': ' + cause.message, {cause});
            }
          }
        `};
      }
      return null;
    }},
    transform: {filters:{moduleTypes:['js','jsx','ts','tsx']}, executor({content, resolvedPath}) {
      if (resolvedPath === runtimePath || resolvedPath === containerPath || resolvedPath.includes('/node_modules/')) return null;
      return rewriteRemoteImports(content, resolvedPath, Object.keys(remotes));
    }}
  };
}
