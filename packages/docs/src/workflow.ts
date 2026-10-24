import { createHash } from 'node:crypto';
import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import type { FarmCliOptions, UserConfig, WorkflowContext } from '@farmfe/core';
import {
  compileDocsTarget,
  compilerNativeIdentity,
  prepareDocsSnapshot,
  resolveConfig
} from '@farmfe/core';
import docs from '@farmfe/plugin-docs';
import seo from '@farmfe/plugin-docs-seo';
import snapshot from '@farmfe/plugin-docs-snapshot';
import react from '@farmfe/plugin-react';
import ssrCss from '@farmfe/plugin-ssr-css';
import { isolateDocsTargetCache } from './build-cache.js';
import type {
  DocPlan,
  NotFoundPlan,
  PublicPage,
  SiteConfig
} from './config.generated.js';
import { startDocsDevelopment } from './development.js';
import { resolveDocsCodeTheme } from './highlight-config.js';
import type { DocsFarmConfig } from './index.js';
import { assertNativePlugins } from './index.js';
import { validateMachineReadableResources } from './machine-readable.js';
import { withOutputTransaction } from './output-transaction.js';
import { startDocsPreview } from './preview.js';
import { withFrameworkAliases } from './resolve-alias.js';
import { renderPages } from './ssg-pool.js';

interface Manifest {
  schemaVersion: number;
  snapshotId: string;
  resources: { name: string; type: string }[];
  template?: string | null;
  pageResources?: Record<
    string,
    { styles: string[]; scripts: string[] }
  > | null;
  machineReadable?: unknown;
}
type WorkflowPlan = Pick<
  DocPlan,
  | 'snapshotId'
  | 'site'
  | 'seo'
  | 'routes'
  | 'notFound'
  | 'ssg'
  | 'publicDiagnostics'
>;
const escapeHtml = (value: string) =>
  value.replace(
    /[&<>"']/g,
    (char) =>
      ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[
        char
      ] ?? char
  );
const jsonForHtml = (value: unknown) =>
  JSON.stringify(value)
    .replace(/</g, '\\u003c')
    .replace(/\u2028/g, '\\u2028')
    .replace(/\u2029/g, '\\u2029');

function checkCancellation(signal?: AbortSignal): void {
  if (signal?.aborted)
    throw Object.assign(new Error('[DOCS_CANCELLED] build cancelled'), {
      cause: signal.reason
    });
}

const fileDigest = (bytes: string | Buffer) =>
  createHash('sha256').update(bytes).digest('hex');

async function renameDirectory(
  source: string,
  destination: string
): Promise<void> {
  const waits = [25, 50, 100, 200, 400, 800];
  for (let attempt = 0; ; attempt++) {
    try {
      await fs.rename(source, destination);
      return;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (
        process.platform !== 'win32' ||
        !code ||
        !['EPERM', 'EACCES', 'EBUSY'].includes(code) ||
        attempt >= waits.length
      )
        throw error;
      console.warn(
        `[DOCS_OUTPUT_RETRY] rename ${code}, attempt ${attempt + 1}/7; retry in ${waits[attempt]}ms: ${source} -> ${destination}`
      );
      await delay(waits[attempt]);
    }
  }
}

function inside(parent: string, child: string): boolean {
  const relative = path.relative(parent, child);
  return (
    relative !== '' &&
    relative !== '..' &&
    !relative.startsWith(`..${path.sep}`) &&
    !path.isAbsolute(relative)
  );
}
function outputFile(directory: string, relative: string): string {
  const file = path.resolve(directory, relative);
  if (
    !relative ||
    relative.includes('\\') ||
    path.isAbsolute(relative) ||
    !inside(directory, file)
  )
    throw new Error(`[DOCS_OUTPUT_PATH] ${relative}`);
  return file;
}
async function assertRealDirectories(
  root: string,
  directory: string
): Promise<void> {
  let current = root;
  for (const segment of path.relative(root, directory).split(path.sep)) {
    current = path.join(current, segment);
    const stat = await fs.lstat(current).catch((error) => {
      if (error.code === 'ENOENT') return undefined;
      throw error;
    });
    if (stat && (stat.isSymbolicLink() || !stat.isDirectory()))
      throw new Error(
        `[DOCS_OUTPUT_PATH] expected a real directory: ${current}`
      );
  }
}
async function validateOutput(root: string, output: string): Promise<boolean> {
  await assertRealDirectories(root, output);
  const files = await fs.readdir(output).catch((error) => {
    if (error.code === 'ENOENT') return undefined;
    throw error;
  });
  if (files?.length) {
    const report = await fs
      .readFile(path.join(output, 'docs-build-report.json'), 'utf8')
      .then((text) => JSON.parse(text))
      .catch(() => undefined);
    if (
      report?.schemaVersion !== 2 ||
      typeof report.snapshotId !== 'string' ||
      !/^[a-f0-9]{64}$/.test(report.snapshotId) ||
      !Array.isArray(report.pages)
    )
      throw new Error(
        '[DOCS_OUTPUT_OWNERSHIP] nonempty output is not a previous Farm Docs build'
      );
  }
  return files !== undefined;
}
function htmlPage(
  template: string,
  plan: WorkflowPlan,
  page: PublicPage | null,
  body: string,
  styles: string[],
  notFound?: NotFoundPlan
): string {
  const head = page ? plan.seo[page.key] : undefined;
  let presentation: {
    locale: string;
    title: string;
    description: string;
    headHtml: string;
  };
  if (page) {
    if (
      !head ||
      typeof head.html !== 'string' ||
      typeof head.title !== 'string'
    )
      throw new Error(`[DOCS_SEO_HEAD] missing Rust head for ${page.key}`);
    presentation = {
      locale: page.locale,
      title: head.title,
      description: page.description ?? '',
      headHtml: head.html
    };
  } else {
    if (!notFound)
      throw new Error('[DOCS_RENDER_PROTOCOL] missing Rust auxiliary plan');
    presentation = notFound;
  }
  const seoMarker =
    '<meta name="farm-docs-seo-head" content="__FARM_DOCS_SEO_HEAD__">';
  const canonicalMarker =
    '<link rel="canonical" href="__FARM_DOCS_CANONICAL__">';
  if (template.split(seoMarker).length !== 2)
    throw new Error('[DOCS_TEMPLATE] expected exactly one SEO head marker');
  const replacements: Record<string, string> = {
    __FARM_DOCS_LANG__: escapeHtml(presentation.locale),
    __FARM_DOCS_TITLE__: escapeHtml(presentation.title),
    __FARM_DOCS_DESCRIPTION__: escapeHtml(presentation.description),
    [canonicalMarker]: head
      ? `<link rel="canonical" href="${escapeHtml(head.canonicalUrl)}">`
      : '',
    [seoMarker]: presentation.headHtml,
    '"__FARM_DOCS_STATE__"': jsonForHtml(
      page
        ? {
            snapshotId: plan.snapshotId,
            key: page.key
          }
        : {
            snapshotId: plan.snapshotId,
            kind: 'notFound',
            locale: presentation.locale
          }
    ),
    __FARM_DOCS_BODY__: body
  };
  for (const token of Object.keys(replacements)) {
    if (template.split(token).length !== 2)
      throw new Error(`[DOCS_TEMPLATE] expected exactly one ${token}`);
  }
  // A single pass prevents rendered content from becoming a second template instruction.
  const html = template.replace(
    /<link rel="canonical" href="__FARM_DOCS_CANONICAL__">|<meta name="farm-docs-seo-head" content="__FARM_DOCS_SEO_HEAD__">|__FARM_DOCS_(?:LANG|TITLE|DESCRIPTION|BODY)__|"__FARM_DOCS_STATE__"/g,
    (token) => replacements[token]
  );
  const marker =
    /<meta name=(?:"farm-docs-page-styles"|farm-docs-page-styles) content=(?:"__FARM_DOCS_STYLE_LINKS__"|__FARM_DOCS_STYLE_LINKS__)>/g;
  if ((html.match(marker) ?? []).length !== 1)
    throw new Error('[DOCS_TEMPLATE] expected exactly one page styles marker');
  const links = styles
    .map(
      (name) =>
        `<link rel="stylesheet" data-farm-docs-page-style href="${escapeHtml(plan.site.basePath + name)}">`
    )
    .join('');
  return html.replace(marker, links);
}

/** Thin orchestration: all content discovery, parsing and routing is native. */
export async function run({
  command,
  config,
  options,
  signal
}: WorkflowContext): Promise<void> {
  checkCancellation(signal);
  if (command === 'dev') {
    const site = options?.site as SiteConfig | undefined;
    if (!site) throw new Error('[DOCS_WORKFLOW] missing site configuration');
    const farm = (options?.farm ?? {}) as DocsFarmConfig;
    await startDocsDevelopment(
      site,
      {
        ...farm,
        root: config.root,
        customLogger: config.customLogger,
        server: { ...farm.server, ...config.server }
      },
      { signal, sourceConfig: config }
    );
    return;
  }
  if (command === 'preview') {
    const farm = (options?.farm ?? {}) as DocsFarmConfig;
    assertNativePlugins(farm);
    await startDocsPreview(
      {
        root: config.root,
        customLogger: config.customLogger,
        server: config.server,
        compilation: {
          output: {
            path:
              config.server?.preview?.distDir ??
              farm.compilation?.output?.path ??
              'build'
          }
        }
      },
      { signal }
    );
    return;
  }
  if (command !== 'build')
    throw new Error(`[DOCS_COMMAND] ${command} is not implemented yet`);
  const inputSite = options?.site as SiteConfig | undefined;
  const farm = (options?.farm ?? {}) as DocsFarmConfig;
  if (!inputSite) throw new Error('[DOCS_WORKFLOW] missing site configuration');
  const { site, highlight: resolvedHighlight } = resolveDocsCodeTheme(
    inputSite,
    farm.snippets
  );
  assertNativePlugins(farm);
  const root = await fs.realpath(config.root ?? process.cwd());
  const { snippets, ...coreFarm } = farm;
  const { default: snippetsPlugin } = await import(
    '@farmfe/plugin-docs-snippets'
  );
  const output = path.resolve(root, farm.compilation?.output?.path ?? 'build');
  if (
    !inside(root, output) ||
    ['.git', 'node_modules', '.farm-docs', '.farm-docs-locks'].includes(
      path.relative(root, output).split(path.sep)[0].toLowerCase()
    )
  )
    throw new Error(
      '[DOCS_OUTPUT_PATH] output must be a dedicated directory inside the site'
    );
  // Check ancestors too: a nested Windows junction can redirect an otherwise lexical child.
  await withOutputTransaction(
    {
      root,
      output,
      signal,
      validateOutput,
      assertRealDirectories,
      renameDirectory
    },
    async (transaction) => {
      const { staging, destination } = transaction;
      const require = createRequire(path.join(root, 'package.json'));
      const frameworkRequire = createRequire(import.meta.url);
      const aliases = {
        react: path.dirname(require.resolve('react/package.json')),
        'react-dom': path.dirname(require.resolve('react-dom/package.json')),
        '@farmfe/docs/client$': fileURLToPath(
          new URL('./client.js', import.meta.url)
        ),
        '@farmfe/docs/snippets$': frameworkRequire.resolve(
          '@farmfe/docs/snippets'
        ),
        '@farmfe/docs/snippets-runtime$': frameworkRequire.resolve(
          '@farmfe/docs/snippets-runtime'
        ),
        '@farmfe/docs-theme-default$': frameworkRequire.resolve(
          '@farmfe/docs-theme-default'
        ),
        '@farmfe/docs-theme-default/style.css$': frameworkRequire.resolve(
          '@farmfe/docs-theme-default/style.css'
        ),
        '@farmfe/plugin-syntax-highlight/style.css$': frameworkRequire.resolve(
          '@farmfe/plugin-syntax-highlight/style.css'
        )
      };
      const planPath = path.join(staging, 'plan.json');
      const nativeSnapshot = await prepareDocsSnapshot(site, root, planPath);
      try {
        const plan: WorkflowPlan = JSON.parse(nativeSnapshot.metadata());
        const nativeHashes = new Map<string, Promise<string>>();
        let coreIdentity: ReturnType<typeof compilerNativeIdentity> | undefined;
        const compile = async (
          pluginOptions: Parameters<typeof docs>[0],
          input: string,
          target: 'node' | 'browser',
          directory: string,
          entryName: string,
          capture?: Parameters<typeof snapshot>[0]
        ) => {
          checkCancellation(signal);
          const childConfig: UserConfig & FarmCliOptions = {
            ...coreFarm,
            root,
            configFile: false,
            compilation: {
              ...farm.compilation,
              // Normalization mutates cache options; targets must not share them.
              persistentCache:
                farm.compilation?.persistentCache &&
                typeof farm.compilation.persistentCache === 'object'
                  ? structuredClone(farm.compilation.persistentCache)
                  : farm.compilation?.persistentCache,
              input: { [entryName]: input },
              output: {
                ...farm.compilation?.output,
                path: directory,
                targetEnv: target,
                format: target === 'node' ? 'cjs' : 'esm',
                entryFilename:
                  target === 'node' ? '[entryName].cjs' : '[entryName].[ext]',
                publicPath: site.basePath ?? '/'
              },
              resolve: {
                ...farm.compilation?.resolve,
                alias: withFrameworkAliases(
                  farm.compilation?.resolve?.alias,
                  aliases
                )
              },
              lazyCompilation: false,
              sourcemap: farm.compilation?.sourcemap ?? false
            },
            plugins: [
              ...(capture ? [snapshot(capture)] : []),
              ...(target === 'browser' && 'planPath' in pluginOptions
                ? [seo({ planPath: pluginOptions.planPath })]
                : []),
              docs(pluginOptions),
              snippetsPlugin({
                ...snippets,
                highlight: resolvedHighlight
              }),
              react({ development: false, refresh: false }),
              ...(target === 'node' ? [ssrCss()] : []),
              ...(farm.plugins ?? [])
            ]
          };
          const resolved = await resolveConfig(
            childConfig,
            'build',
            'production'
          );
          checkCancellation(signal);
          if (
            resolved.jsPlugins?.some(
              (plugin) =>
                plugin.name !== 'farm:external-adapter' ||
                Object.keys(plugin).some(
                  (key) => !['name', 'priority', 'config'].includes(key)
                )
            )
          )
            throw new Error('[DOCS_RUST_ONLY] compilation hooks must be Rust');
          if (
            resolved.compilation?.persistentCache &&
            typeof resolved.compilation.persistentCache === 'object'
          ) {
            coreIdentity ??= compilerNativeIdentity();
            const roles = new Map([
              [planPath, 'farm-docs-transaction://plan'],
              [directory, `farm-docs-transaction://${target}-output`],
              ...(capture?.mode === 'capture'
                ? ([
                    [
                      capture.receiptPath,
                      `farm-docs-transaction://${target}-receipt`
                    ],
                    [capture.seedPath, `farm-docs-transaction://${target}-seed`]
                  ] as [string, string][])
                : [])
            ]);
            await isolateDocsTargetCache(resolved, {
              target,
              entryName,
              snapshotId: plan.snapshotId,
              site,
              nativeIdentity: await coreIdentity,
              roles,
              nativeHashes
            });
          }
          for (let attempt = 0; ; attempt++) {
            checkCancellation(signal);
            try {
              // Native compilation has no cancellation API: await release before cleanup.
              const result = await compileDocsTarget(resolved, nativeSnapshot);
              checkCancellation(signal);
              if (capture?.mode === 'capture') {
                const receipt = JSON.parse(
                  await fs.readFile(capture.receiptPath, 'utf8')
                );
                receipt.schemaVersion = 2;
                receipt.inputObservations = result.inputObservations;
                await fs.writeFile(
                  capture.receiptPath,
                  JSON.stringify(receipt)
                );
              }
              return result.resources;
            } catch (error) {
              checkCancellation(signal);
              if (
                !capture ||
                capture.mode !== 'capture' ||
                attempt >= 2 ||
                !(error instanceof Error) ||
                error.message !==
                  '[DOCS_INPUT_DISCOVERY] newly discovered watch inputs require an observed retry'
              )
                throw error;
              console.warn(
                `[DOCS_INPUT_DISCOVERY] observing newly registered inputs; retry ${attempt + 1}/2`
              );
            }
          }
        };
        checkCancellation(signal);
        const browserReceipt = path.join(staging, 'browser-receipt.json');
        const serverReceipt = path.join(staging, 'server-receipt.json');
        const serverDirectory = path.join(staging, 'server');
        const compileBrowser = () =>
          compile(
            { planPath },
            fileURLToPath(new URL('../client.html', import.meta.url)),
            'browser',
            destination,
            'app',
            {
              mode: 'capture',
              planPath,
              receiptPath: browserReceipt,
              seedPath: path.join(staging, 'browser-seed.json'),
              target: 'browser',
              outputPath: output
            }
          );
        const compileServer = () =>
          compile(
            { planPath },
            fileURLToPath(new URL('./server-entry.js', import.meta.url)),
            'node',
            serverDirectory,
            'server',
            {
              mode: 'capture',
              planPath,
              receiptPath: serverReceipt,
              seedPath: path.join(staging, 'server-seed.json'),
              target: 'server',
              outputPath: output
            }
          );
        let browser: Record<string, Buffer>;
        let server: Record<string, Buffer>;
        if (site.build?.targetConcurrency === 2) {
          // Explicit opt-in: two independent target graphs can increase peak memory.
          // Drain both native calls before failure permits transaction cleanup.
          const results = await Promise.allSettled([
            compileBrowser(),
            compileServer()
          ]);
          const failed = results.find((result) => result.status === 'rejected');
          if (failed?.status === 'rejected') throw failed.reason;
          const [browserResult, serverResult] = results;
          if (
            browserResult.status !== 'fulfilled' ||
            serverResult.status !== 'fulfilled'
          )
            throw new Error('[DOCS_TARGETS] incomplete target results');
          browser = browserResult.value;
          server = serverResult.value;
        } else {
          browser = await compileBrowser();
          server = await compileServer();
        }
        const browserHash = fileDigest(await fs.readFile(browserReceipt));
        const serverHash = fileDigest(await fs.readFile(serverReceipt));
        const manifests = [browser, server].map((resources) => {
          if (!resources['docs-manifest.json'])
            throw new Error('[DOCS_MANIFEST] native manifest missing');
          const manifest: Manifest = JSON.parse(
            resources['docs-manifest.json'].toString()
          );
          if (
            manifest.schemaVersion !== 1 ||
            manifest.snapshotId !== plan.snapshotId
          )
            throw new Error('[DOCS_MANIFEST] snapshot mismatch');
          for (const resource of manifest.resources) {
            outputFile(destination, resource.name);
            if (!resources[resource.name])
              throw new Error(
                `[DOCS_MANIFEST] resource missing: ${resource.name}`
              );
          }
          return manifest;
        });
        const templateName = manifests[0].template;
        validateMachineReadableResources(
          manifests[0].machineReadable,
          {
            snapshotId: plan.snapshotId,
            basePath: plan.site.basePath,
            enabled: plan.site.ai != null && plan.site.ai.enabled !== false,
            publicationStatus: plan.site.ai?.publicationStatus ?? 'prerelease',
            pages: plan.routes.map(({ page }) => ({
              ...page,
              canonicalUrl: plan.seo[page.key]?.canonicalUrl
            }))
          },
          browser,
          manifests[0].resources.map(({ name }) => name)
        );
        if (!templateName || !browser[templateName])
          throw new Error('[DOCS_MANIFEST] native HTML template missing');
        const template = browser[templateName].toString();
        const templatePath = path.join(staging, 'browser-template.html');
        await fs.writeFile(templatePath, browser[templateName]);
        const pageResources = manifests[0].pageResources;
        if (!pageResources)
          throw new Error('[DOCS_MANIFEST] native page resources missing');
        for (const route of plan.routes) {
          const resources = pageResources[route.page.key];
          if (
            !resources ||
            !Array.isArray(resources.styles) ||
            !Array.isArray(resources.scripts)
          )
            throw new Error('[DOCS_MANIFEST] page resource lists missing');
          for (const name of [...resources.styles, ...resources.scripts]) {
            outputFile(destination, name);
            if (!browser[name])
              throw new Error(`[DOCS_MANIFEST] page resource missing: ${name}`);
          }
        }
        for (const [directory, resources] of [
          [destination, browser],
          [serverDirectory, server]
        ] as const) {
          for (const [name, bytes] of Object.entries(resources)) {
            checkCancellation(signal);
            if (directory === destination && name === templateName) continue;
            if (name === 'docs-plan.json')
              throw new Error(
                '[DOCS_PRIVATE_PLAN] child output contains private plan'
              );
            const file = outputFile(directory, name);
            await fs.mkdir(path.dirname(file), { recursive: true });
            await fs.writeFile(file, bytes);
          }
        }
        // Farm CJS chunks use .js filenames. Scope their Node interpretation to this private build.
        await fs.writeFile(
          path.join(serverDirectory, 'package.json'),
          '{"type":"commonjs"}\n'
        );
        const writtenPages: {
          key: string;
          outputPath: string;
          hash: string;
        }[] = [];
        const writtenNotFound = new Map<
          string,
          { locale: string; outputPath: string; hash: string }
        >();
        await renderPages({
          signal,
          serverEntry: path.join(serverDirectory, 'server.cjs'),
          snapshotId: plan.snapshotId,
          routes: plan.routes,
          notFound: plan.notFound,
          limits: plan.ssg,
          onNotFound: async (notFound, html) => {
            const file = outputFile(destination, notFound.outputPath);
            if (browser[notFound.outputPath])
              throw new Error(`[DOCS_OUTPUT_COLLISION] ${notFound.outputPath}`);
            const page = htmlPage(template, plan, null, html, [], notFound);
            await fs.mkdir(path.dirname(file), { recursive: true });
            await fs.writeFile(file, page);
            if (writtenNotFound.has(notFound.locale))
              throw new Error(
                '[DOCS_RENDER_PROTOCOL] duplicate NotFound output'
              );
            writtenNotFound.set(notFound.locale, {
              locale: notFound.locale,
              outputPath: notFound.outputPath,
              hash: fileDigest(page)
            });
          },
          onPage: async (route, html) => {
            const file = outputFile(destination, route.outputPath);
            if (browser[route.outputPath])
              throw new Error(`[DOCS_OUTPUT_COLLISION] ${route.outputPath}`);
            await fs.mkdir(path.dirname(file), { recursive: true });
            const page = htmlPage(
              template,
              plan,
              route.page,
              html,
              pageResources[route.page.key].styles
            );
            await fs.writeFile(file, page);
            writtenPages.push({
              key: route.page.key,
              outputPath: route.outputPath,
              hash: fileDigest(page)
            });
          }
        });
        if (writtenNotFound.size !== plan.notFound.length)
          throw new Error('[DOCS_RENDER_PROTOCOL] missing NotFound output');
        checkCancellation(signal);
        const report = JSON.stringify(
          {
            schemaVersion: 2,
            snapshotId: plan.snapshotId,
            pages: plan.routes.map((route) => ({
              key: route.page.key,
              routePath: route.page.routePath,
              outputPath: route.outputPath
            })),
            notFound: plan.notFound.map(
              ({ locale, routeBase, outputPath }) => ({
                locale,
                routeBase,
                outputPath
              })
            ),
            preview: { basePath: plan.site.basePath },
            diagnostics: plan.publicDiagnostics
          },
          null,
          2
        );
        await fs.writeFile(
          path.join(destination, 'docs-build-report.json'),
          report
        );
        const writesPath = path.join(staging, 'written-files.json');
        const writes = JSON.stringify({
          schemaVersion: 3,
          snapshotId: plan.snapshotId,
          pages: writtenPages,
          notFound: plan.notFound.map((page) =>
            writtenNotFound.get(page.locale)
          ),
          reportHash: fileDigest(report)
        });
        await fs.writeFile(writesPath, writes);
        checkCancellation(signal);
        await nativeSnapshot.verify(
          JSON.stringify({
            browserReceipt,
            browserHash,
            serverReceipt,
            serverHash,
            writesPath,
            writesHash: fileDigest(writes),
            publicDirectory: destination,
            serverDirectory,
            templatePath
          })
        );
        checkCancellation(signal);
        await transaction.commit();
        console.log(
          `Farm Docs: generated ${plan.routes.length} pages in ${path.relative(root, output)} (${plan.snapshotId.slice(0, 12)})`
        );
      } finally {
        // All native compile/verify work is awaited before releasing ownership
        // or permitting transaction cleanup.
        nativeSnapshot.close();
      }
    }
  );
}
