import type {
  CompilationContext,
  JsPlugin,
  Server,
  UserConfig
} from '@farmfe/core';
import { compile } from '@tailwindcss/node';
import { clearRequireCache } from '@tailwindcss/node/require-cache';

import fs from 'node:fs/promises';
import path from 'path';
import { Scanner } from '@tailwindcss/oxide';
import { Features, transform } from 'lightningcss';
import postcss from 'postcss';
import postcssImport from 'postcss-import';

// like https://github.com/tailwindlabs/tailwindcss/blob/next/packages/%40tailwindcss-vite/src/index.ts
export default function tailwindcss(): JsPlugin[] {
  let servers: Server[] = [];
  let config: UserConfig | null = null;

  let isSSR = false;
  let minify = false;

  let moduleGraphCandidates = new Set<string>();
  let moduleGraphScanner = new Scanner({});

  let roots: DefaultMap<string, Root> = new DefaultMap(
    (id) => new Root(id, () => moduleGraphCandidates, config!.root!)
  );

  function scanFile(
    _id: string,
    content: string,
    extension: string,
    isSSR: boolean,
    ctx: CompilationContext | undefined
  ) {
    let updated = false;
    for (let candidate of moduleGraphScanner.scanFiles([
      { content, extension }
    ])) {
      updated = true;
      moduleGraphCandidates.add(candidate);
    }

    if (updated) {
      invalidateAllRoots(isSSR, ctx);
    }
  }

  function invalidateAllRoots(
    isSSR: boolean,
    ctx: CompilationContext | undefined
  ) {
    for (let server of servers) {
      for (let id of roots.keys()) {
        let isAlive = server.getCompiler().hasModule(id);
        if (!isAlive) {
          // Note: Removing this during SSR is not safe and will produce
          // inconsistent results based on the timing of the removal and
          // the order / timing of transforms.
          if (!isSSR) {
            // It is safe to remove the item here since we're iterating on a copy
            // of the keys.
            roots.delete(id);
          }
          continue;
        }

        roots.get(id).requiresRebuild = false;
        server.hmrEngine?.hmrUpdate(id, true);
      }
    }
  }

  async function regenerateOptimizedCss(
    root: Root,
    addWatchFile: (file: string) => void
  ) {
    let content = root.lastContent;
    let generated = await root.generate(content, addWatchFile);
    if (generated === false) {
      return;
    }
    return optimizeCss(generated, { minify });
  }

  return [
    {
      name: 'farm:tailwindcss:scan',
      priority: 98,
      config(_config) {
        config = _config;
        minify = !!config.compilation?.minify;

        return {
          compilation: {
            // TODO: should invalidate entry css file when config changes
            persistentCache: false
          }
        };
      },
      configureDevServer(server) {
        servers.push(server);
      },
      transformHtml: {
        executor(param, ctx) {
          if (param.htmlResource?.info) {
            scanFile(
              param.htmlResource.info?.id,
              bytes2String(param.htmlResource.bytes),
              'html',
              isSSR,
              ctx
            );
          }
          return undefined;
        }
      },
      transform: {
        filters: {
          resolvedPaths: ['.+']
        },
        executor(param, context) {
          let extension = getExtension(param.resolvedPath);
          if (isPotentialCssRootFile(param.resolvedPath)) return;
          scanFile(
            param.resolvedPath,
            param.content,
            extension,
            isSSR,
            context
          );
          return undefined;
        }
      }
    },

    {
      name: 'farm:tailwindcss:post',
      priority: 100,
      transform: {
        filters: {
          resolvedPaths: ['.+']
        },
        async executor(param, context) {
          if (!isPotentialCssRootFile(param.resolvedPath)) return;
          let root = roots.get(param.resolvedPath);

          // We do a first pass to generate valid CSS for the downstream plugins.
          // However, since not all candidates are guaranteed to be extracted by
          // this time, we have to re-run a transform for the root later.
          let generated = await root.generate(param.content, (file) =>
            context?.addWatchFile(param.resolvedPath, file)
          );

          if (!generated) {
            roots.delete(param.resolvedPath);
            return undefined;
          }

          return { content: generated, moduleType: param.moduleType };
        }
      },
      renderStart: {
        async executor() {
          for (let [id, root] of roots.entries()) {
            let generated = await regenerateOptimizedCss(
              root,
              // During the renderStart phase, we can not add watch files since
              // those would not be causing a refresh of the right CSS file. This
              // should not be an issue since we did already process the CSS file
              // before and the dependencies should not be changed (only the
              // candidate list might have)
              () => {}
            );
            if (!generated) {
              roots.delete(id);
              continue;
            }

            // These plugins have side effects which, during build, results in CSS
            // being written to the output dir. We need to run them here to ensure
            // the CSS is written before the bundle is generated.
            // await transformWithPlugins(this, id, generated);
          }
          return undefined;
        }
      }
    }
  ];
}

function getExtension(id: string) {
  let [filename] = id.split('?', 2);
  return path.extname(filename).slice(1);
}

function isPotentialCssRootFile(id: string) {
  let extension = getExtension(id);
  let isCssFile = extension === 'css';
  return isCssFile;
}

function isCssRootFile(content: string) {
  return (
    content.includes('@tailwind') ||
    content.includes('@config') ||
    content.includes('@plugin') ||
    content.includes('@apply') ||
    content.includes('@theme') ||
    content.includes('@variant') ||
    content.includes('@utility')
  );
}

function optimizeCss(
  input: string,
  {
    file = 'input.css',
    minify = false
  }: { file?: string; minify?: boolean } = {}
) {
  return transform({
    filename: file,
    code: Buffer.from(input),
    minify,
    sourceMap: false,
    drafts: {
      customMedia: true
    },
    nonStandard: {
      deepSelectorCombinator: true
    },
    include: Features.Nesting,
    exclude: Features.LogicalProperties,
    targets: {
      safari: (16 << 16) | (4 << 8)
    },
    errorRecovery: true
  }).code.toString();
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
  // Content is only used in serve mode where we need to capture the initial
  // contents of the root file so that we can restore it during the
  // `renderStart` hook.
  public lastContent = '';

  // The lazily-initialized Tailwind compiler components. These are persisted
  // throughout rebuilds but will be re-initialized if the rebuild strategy is
  // set to `full`.
  private compiler?: Awaited<ReturnType<typeof compile>>;

  public requiresRebuild = true;

  // This is the compiler-specific scanner instance that is used only to scan
  // files for custom @source paths. All other modules we scan for candidates
  // will use the shared moduleGraphScanner instance.
  private scanner?: Scanner;

  // List of all candidates that were being returned by the root scanner during
  // the lifetime of the root.
  private candidates: Set<string> = new Set<string>();

  // List of all file dependencies that were captured while generating the root.
  // These are retained so we can clear the require cache when we rebuild the
  // root.
  private dependencies = new Set<string>();

  constructor(
    private id: string,
    private getSharedCandidates: () => Set<string>,
    private base: string
  ) {}

  // Generate the CSS for the root file. This can return false if the file is
  // not considered a Tailwind root. When this happened, the root can be GCed.
  public async generate(
    content: string,
    addWatchFile: (file: string) => void
  ): Promise<string | false> {
    this.lastContent = content;

    let inputPath = idToPath(this.id);
    let inputBase = path.dirname(path.resolve(inputPath));

    if (!this.compiler || !this.scanner || this.requiresRebuild) {
      clearRequireCache(Array.from(this.dependencies));
      this.dependencies = new Set([idToPath(inputPath)]);

      let postcssCompiled = await postcss([
        postcssImport({
          load: (path) => {
            this.dependencies.add(path);
            addWatchFile(path);
            return fs.readFile(path, 'utf8');
          }
        })
        // fixRelativePathsPlugin()
      ]).process(content, {
        from: inputPath,
        to: inputPath
      });
      let css = postcssCompiled.css;

      // This is done inside the Root#generate() method so that we can later use
      // information from the Tailwind compiler to determine if the file is a
      // CSS root (necessary because we will probably inline the `@import`
      // resolution at some point).
      if (!isCssRootFile(css)) {
        return false;
      }

      this.compiler = await compile(css, {
        base: inputBase,
        onDependency: (path) => {
          addWatchFile(path);
          this.dependencies.add(path);
        }
      });

      this.scanner = new Scanner({
        sources: this.compiler.globs.map(({ origin, pattern }) => ({
          // Ensure the glob is relative to the input CSS file or the config
          // file where it is specified.
          base: origin
            ? path.dirname(path.resolve(inputBase, origin))
            : inputBase,
          pattern
        }))
      });
    }

    // This should not be here, but right now the Vite plugin is setup where we
    // setup a new scanner and compiler every time we request the CSS file
    // (regardless whether it actually changed or not).
    for (let candidate of this.scanner.scan()) {
      this.candidates.add(candidate);
    }

    // Watch individual files found via custom `@source` paths
    for (let file of this.scanner.files) {
      addWatchFile(file);
    }

    // Watch globs found via custom `@source` paths
    for (let glob of this.scanner.globs) {
      if (glob.pattern[0] === '!') continue;

      let relative = path.relative(this.base, glob.base);
      if (relative[0] !== '.') {
        relative = './' + relative;
      }
      // Ensure relative is a posix style path since we will merge it with the
      // glob.
      // relative = normalizePath(relative);

      addWatchFile(path.posix.join(relative, glob.pattern));
    }

    this.requiresRebuild = true;

    return this.compiler.build([
      ...this.getSharedCandidates(),
      ...this.candidates
    ]);
  }
}

function bytes2String(bytes: number[]): string {
  return new TextDecoder().decode(new Uint8Array(bytes));
}
