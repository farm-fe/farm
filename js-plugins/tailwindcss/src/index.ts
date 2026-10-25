import fs from 'node:fs/promises';
import path from 'node:path';
/**
 * MIT License Copied and modified by BrightWu
 * Copyright Farm and All Contributors
 * Copyright (c) Tailwind Labs, Inc.
 */
import { JsPlugin, logger, ResolvedUserConfig, Resolver } from '@farmfe/core';
import {
  compile,
  env,
  Features,
  Instrumentation,
  optimize,
  toSourceMap
} from '@tailwindcss/node';
import { clearRequireCache } from '@tailwindcss/node/require-cache';
import { Scanner } from '@tailwindcss/oxide';

const DEBUG = env.DEBUG;
const SPECIAL_QUERY_RE = /[?&](?:worker|sharedworker|raw|url)\b/;
const COMMON_JS_PROXY_RE = /\?commonjs-proxy/;
const INLINE_STYLE_ID_RE = /[?&]index\=\d+\.css$/;

export interface Options {
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
}

const CANDIDATE_NAME = 'tailwindcss:candidate';
const CANDIDATE_SCOPE = 'tailwindcss:candidateScope';

export default function tailwindcss(options: Options = {}): JsPlugin[] {
  let config: ResolvedUserConfig | null = null;
  let minify = false;

  let roots: DefaultMap<string, Root> = new DefaultMap((id) => {
    const compilation = config?.compilation ?? {};

    const cssResolver = new Resolver({
      ...compilation,
      resolve: {
        ...(compilation.resolve ?? {}),
        extensions: ['.css'],
        mainFields: ['style'],
        conditions: ['style', 'development|production']
      }
    });
    function customCssResolver(id: string, base: string) {
      return Promise.resolve(cssResolver.resolve(id, base));
    }

    const jsResolver = new Resolver(config?.compilation ?? {});
    function customJsResolver(id: string, base: string) {
      return Promise.resolve(jsResolver.resolve(id, base));
    }
    return new Root(
      id,
      config?.root || process.cwd(),
      // Currently, Vite only supports CSS source maps in development and they
      // are off by default. Check to see if we need them or not.
      config?.compilation?.sourcemap
        ? Boolean(config.compilation.sourcemap)
        : false,
      customCssResolver,
      customJsResolver
    );
  });

  // Tailwind scanner
  const scanner = new Scanner({});
  // List of all candidates that were being returned by the root scanner during
  // the lifetime of the root.
  // const candidatesMap: Map<string, Set<string>> = new Map();
  const candidatesModuleList = new Map<string, boolean>();

  return [
    {
      // Step 1: Scan source files for candidates
      name: '@farmfe/js-plugin-tailwindcss:scan',
      priority: 101,

      async configResolved(_config) {
        config = _config;
        minify = config.compilation?.minify !== false;
      },

      pluginCacheLoaded: {
        executor(cache) {
          const sources: [string, boolean][] = JSON.parse(
            Buffer.from(cache).toString('utf8')
          );
          for (const [source, physical] of sources) {
            candidatesModuleList.set(source, physical);
          }
        }
      },

      writePluginCache: {
        executor() {
          // Source transforms are skipped on cache hits, so keep their watch edges
          // available when an unchanged source and a changed stylesheet are rebuilt.
          return [...Buffer.from(JSON.stringify([...candidatesModuleList]))];
        }
      },

      transform: {
        filters: options?.filters ?? { resolvedPaths: ['!node_modules/'] },
        async executor(param, context) {
          const candidateList = [
            ...scanner.scanFiles([
              {
                content: param.content,
                extension: getExtension(param.resolvedPath)
              }
            ])
          ];

          const candidate =
            context?.readMetadata<string[]>(CANDIDATE_NAME, {
              refer: [param.resolvedPath],
              scope: [CANDIDATE_SCOPE]
            }) ?? [];

          context?.writeMetadata(
            param.resolvedPath,
            [...new Set(candidateList.concat(candidate))],
            {
              refer: [param.resolvedPath],
              scope: [CANDIDATE_SCOPE]
            }
          );

          const physical = await fs.stat(param.resolvedPath.split('?')[0]).then(
            (stat) => stat.isFile(),
            () => false
          );
          candidatesModuleList.set(param.resolvedPath, physical);

          return {
            content: param.content
          };
        }
      }
    },

    {
      // Step 2 (serve mode): Generate CSS
      name: '@farmfe/js-plugin-tailwindcss:generate',
      priority: 101,

      pluginCacheLoaded: {
        executor(bytes) {
          const state = JSON.parse(Buffer.from(bytes).toString('utf8'));
          if (
            state?.version !== 1 ||
            !Array.isArray(state.roots) ||
            !state.roots.every((id: unknown) => typeof id === 'string')
          ) {
            throw new Error('Invalid Tailwind plugin cache');
          }
          // Scan metadata and physical-source flags have their own cache.
          roots.clear();
          for (const id of state.roots) roots.get(id);
        }
      },
      writePluginCache: {
        executor() {
          return Array.from(
            Buffer.from(
              JSON.stringify({
                version: 1,
                roots: [...roots.keys()].sort()
              })
            )
          );
        }
      },
      updateModules: {
        executor(param) {
          const rootPath = config?.root ?? process.cwd();
          const normalize = (id: string) =>
            path.resolve(rootPath, id).replaceAll('\\', '/');
          const existing = new Map(
            param.paths.map(([id, type]) => [normalize(id), type])
          );
          for (const id of roots.keys()) {
            if (existing.get(normalize(id)) === 'removed') roots.delete(id);
          }
          const hasCandidateUpdate = param.paths.some(([id]) => {
            const source = id.replaceAll('\\', '/');
            return (
              !source.includes('/node_modules/') &&
              (source.includes('.farm_dynamic_import_virtual_module') ||
                /\.(?:js|jsx|ts|tsx|vue|svelte|html|mdx|md|astro|php|twig|erb|hbs|liquid|pug|slim|haml)(?:\?|$)/.test(
                  source
                ))
            );
          });
          if (!hasCandidateUpdate) return;
          const next = [...param.paths];
          for (const id of roots.keys()) {
            if (!existing.has(normalize(id)))
              next.push([path.resolve(rootPath, id), 'updated']);
          }
          return next;
        }
      },

      freezeModule: {
        filters: { resolvedPaths: ['\\.css'] },
        async executor(param, context) {
          if (!isPotentialCssRootFile(param.moduleId)) return;

          const I = new Instrumentation();
          DEBUG && I.start('[@farmfe/js-plugin-tailwindcss] Generate CSS');

          let root = roots.get(param.moduleId);

          // add watch file for scanned files
          const candidateList: string[] = [];
          for (const [resolvedPath, physical] of candidatesModuleList) {
            if (physical) {
              try {
                await fs.stat(resolvedPath.split('?')[0]);
              } catch {
                candidatesModuleList.delete(resolvedPath);
                continue;
              }
            }

            candidateList.push(
              ...(context?.readMetadata<string[]>(resolvedPath, {
                refer: [resolvedPath],
                scope: [CANDIDATE_SCOPE]
              }) ?? [])
            );
          }
          let result = await root.generate(
            param.content,
            (file) => context?.addWatchFile?.(param.moduleId, file),
            I,
            new Set(candidateList)
          );
          if (!result) {
            roots.delete(param.moduleId);
            return {
              content: param.content
            };
          }
          // Ordinary CSS must not inherit every candidate's watch edge.
          for (const [resolvedPath, physical] of candidatesModuleList) {
            context?.addWatchFile(
              param.moduleId,
              physical ? resolvedPath.split('?')[0] : resolvedPath
            );
          }
          DEBUG && I.end('[@farmfe/js-plugin-tailwindcss] Generate CSS');

          DEBUG && I.start('[@farmfe/js-plugin-tailwindcss] Optimize CSS');
          result = optimize(result.code, {
            minify,
            map: result.map
          });
          DEBUG && I.end('[@farmfe/js-plugin-tailwindcss] Optimize CSS');

          return typeof result === 'string'
            ? {
                content: result
              }
            : {
                content: result.code,
                sourceMap: result.map
              };
        }
      }
    }
  ] satisfies JsPlugin[];
}

function getExtension(id: string) {
  let [filename] = id.split('?', 2);
  return path.extname(filename).slice(1);
}

function isPotentialCssRootFile(id: string) {
  if (id.includes('/.vite/')) return;
  let extension = getExtension(id);
  let isCssFile =
    (extension === 'css' ||
      id.includes('&lang.css') ||
      id.match(INLINE_STYLE_ID_RE)) &&
    // Don't intercept special static asset resources
    !SPECIAL_QUERY_RE.test(id) &&
    !COMMON_JS_PROXY_RE.test(id);
  return isCssFile;
}

function idToPath(id: string) {
  return path.resolve(id.replace(/\?.*$/, ''));
}

/**
 * A Map that can generate default values for keys that don't exist.
 * Generated default values are added to the map to avoid recomputation.
 */
class DefaultMap<K, V> extends Map<K, V> {
  constructor(private factory: (key: K, self: DefaultMap<K, V>) => V) {
    super();
  }

  get(key: K): V {
    let value = super.get(key);

    if (value === undefined) {
      value = this.factory(key, this);
      this.set(key, value);
    }

    return value;
  }
}

class Root {
  // The lazily-initialized Tailwind compiler components. These are persisted
  // throughout rebuilds but will be re-initialized if the rebuild strategy is
  // set to `full`.
  private compiler?: Awaited<ReturnType<typeof compile>>;

  // List of all build dependencies (e.g. imported  stylesheets or plugins) and
  // their last modification timestamp. If no mtime can be found, we need to
  // assume the file has always changed.
  private buildDependencies = new Map<string, number | null>();

  constructor(
    private id: string,
    private base: string,

    private enableSourceMaps: boolean,
    private customCssResolver: (
      id: string,
      base: string
    ) => Promise<string | false | undefined>,
    private customJsResolver: (
      id: string,
      base: string
    ) => Promise<string | false | undefined>
  ) {}

  // Generate the CSS for the root file. This can return false if the file is
  // not considered a Tailwind root. When this happened, the root can be GCed.
  public async generate(
    content: string,
    _addWatchFile: (file: string) => void,
    I: Instrumentation,
    candidates: Set<string>
  ): Promise<
    | {
        code: string;
        map: string | undefined;
      }
    | false
  > {
    // handle virtual id
    let inputPath = path.isAbsolute(this.id)
      ? idToPath(this.id)
      : idToPath(path.join(this.base, this.id));

    function addWatchFile(file: string) {
      // Don't watch the input file since it's already a dependency anc causes
      // issues with some setups (e.g. Qwik).
      if (file === inputPath) {
        return;
      }

      // Scanning `.svg` file containing a `#` or `?` in the path will
      // crash Vite. We work around this for now by ignoring updates to them.
      //
      // https://github.com/tailwindlabs/tailwindcss/issues/16877
      if (/[\#\?].*\.svg$/.test(file)) {
        return;
      }
      _addWatchFile(file);
    }

    let requiresBuildPromise = this.requiresBuild();
    let inputBase = path.dirname(path.resolve(inputPath));

    if (!this.compiler || (await requiresBuildPromise)) {
      clearRequireCache(Array.from(this.buildDependencies.keys()));
      this.buildDependencies.clear();

      this.addBuildDependency(idToPath(inputPath));

      DEBUG && I.start('Setup compiler');
      let addBuildDependenciesPromises: Promise<void>[] = [];
      this.compiler = await compile(content, {
        from: this.enableSourceMaps ? this.id : undefined,
        base: inputBase,
        shouldRewriteUrls: true,
        onDependency: (path) => {
          addWatchFile(path);
          addBuildDependenciesPromises.push(this.addBuildDependency(path));
        },

        customCssResolver: this.customCssResolver,
        customJsResolver: this.customJsResolver
      });
      await Promise.all(addBuildDependenciesPromises);
      DEBUG && I.end('Setup compiler');
    } else {
      for (let buildDependency of this.buildDependencies.keys()) {
        addWatchFile(buildDependency);
      }
    }

    if (
      !(
        this.compiler.features &
        (Features.AtApply |
          Features.JsPluginCompat |
          Features.ThemeFunction |
          Features.Utilities)
      )
    ) {
      return false;
    }

    DEBUG && I.start('Build CSS');
    let code = this.compiler.build([...candidates]);
    DEBUG && I.end('Build CSS');

    DEBUG && I.start('Build Source Map');
    let map = this.enableSourceMaps
      ? toSourceMap(this.compiler.buildSourceMap()).raw
      : undefined;
    DEBUG && I.end('Build Source Map');

    return {
      code,
      map
    };
  }

  private async addBuildDependency(path: string) {
    let mtime: number | null = null;
    try {
      mtime = (await fs.stat(path)).mtimeMs;
    } catch {}
    this.buildDependencies.set(path, mtime);
  }

  private async requiresBuild(): Promise<boolean> {
    for (let [path, mtime] of this.buildDependencies) {
      if (mtime === null) return true;
      try {
        let stat = await fs.stat(path);
        if (stat.mtimeMs > mtime) {
          return true;
        }
      } catch {
        return true;
      }
    }
    return false;
  }
}
