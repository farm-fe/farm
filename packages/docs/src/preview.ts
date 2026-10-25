import { createReadStream } from 'node:fs';
import fs from 'node:fs/promises';
import type { IncomingMessage, ServerResponse } from 'node:http';
import path from 'node:path';
import { pipeline } from 'node:stream/promises';
import {
  type FarmCliOptions,
  PreviewServer,
  resolveConfig,
  type UserConfig
} from '@farmfe/core';
import mime from 'mime';
import { assertNativePlugins, type DocsFarmConfig } from './index.js';

export interface DocsPreviewHandle {
  url: string;
  close(): Promise<void>;
}
export interface DocsPreviewExecution {
  signal?: AbortSignal;
}

function inside(root: string, file: string): boolean {
  const relative = path.relative(root, file);
  return (
    relative !== '' &&
    !relative.startsWith('..' + path.sep) &&
    relative !== '..' &&
    !path.isAbsolute(relative)
  );
}
function checkAbort(signal?: AbortSignal) {
  if (signal?.aborted)
    throw new Error('[DOCS_PREVIEW_ABORT] preview cancelled');
}
function resourceName(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.length > 0 &&
    !value.startsWith('/') &&
    !/[:\\\x00-\x1f\x7f]/.test(value) &&
    !value
      .split('/')
      .some(
        (part) =>
          !part ||
          part === '.' ||
          part === '..' ||
          ['.git', '.farm-docs', '.farm-docs-locks'].includes(
            part.toLowerCase()
          )
      ) &&
    !['docs-plan.json', 'docs-public-config.json'].includes(value.toLowerCase())
  );
}
function requestPath(raw: string): string | undefined {
  if (
    !raw.startsWith('/') ||
    raw.startsWith('//') ||
    raw.includes('#') ||
    /%(?:2f|5c)/i.test(raw)
  )
    return;
  let decoded: string;
  try {
    decoded = decodeURIComponent(raw);
  } catch {
    return;
  }
  if (
    /[\\\x00-\x1f\x7f]/.test(decoded) ||
    decoded.includes('//') ||
    decoded.split('/').some((part) => part === '.' || part === '..')
  )
    return;
  return decoded;
}

/** Read only regular files beneath the successful output, without symlink traversal. */
async function regularFile(root: string, name: string) {
  if (!resourceName(name))
    throw new Error('[DOCS_PREVIEW_PATH] invalid output member');
  let file = root;
  const parts = name.split('/');
  for (let index = 0; index < parts.length; index++) {
    file = path.join(file, parts[index]);
    const stat = await fs.lstat(file);
    if (
      stat.isSymbolicLink() ||
      (index === parts.length - 1 ? !stat.isFile() : !stat.isDirectory())
    )
      throw new Error(
        '[DOCS_PREVIEW_PATH] output member is not a regular file'
      );
  }
  const real = await fs.realpath(file);
  if (!inside(root, real))
    throw new Error('[DOCS_PREVIEW_PATH] output member escapes the site');
  return { file, stat: await fs.stat(file) };
}

async function artifacts(output: string) {
  const read = async (name: string) =>
    JSON.parse(
      await fs.readFile((await regularFile(output, name)).file, 'utf8')
    );
  const report = await read('docs-build-report.json'),
    manifest = await read('docs-manifest.json');
  const basePath: unknown = report.preview?.basePath;
  if (
    report.schemaVersion !== 2 ||
    manifest.schemaVersion !== 1 ||
    typeof report.snapshotId !== 'string' ||
    !/^[a-f0-9]{64}$/.test(report.snapshotId) ||
    manifest.snapshotId !== report.snapshotId ||
    typeof basePath !== 'string' ||
    !basePath.endsWith('/') ||
    /[\s?#%]/u.test(basePath) ||
    requestPath(basePath) !== basePath ||
    !Array.isArray(report.pages) ||
    !report.pages.length ||
    !Array.isArray(manifest.resources)
  )
    throw new Error(
      '[DOCS_PREVIEW_REPORT] build artifacts are missing or incompatible; build with matching workspace packages'
    );
  type Auxiliary = { locale: string; routeBase: string; outputPath: string };
  const parseAuxiliary = (value: unknown): Auxiliary[] => {
    const invalid = () =>
      new Error('[DOCS_PREVIEW_REPORT] invalid Rust auxiliary table');
    if (!Array.isArray(value) || value.length === 0) throw invalid();
    const locales = new Set<string>(),
      mounts = new Set<string>(),
      outputs = new Set<string>();
    return value.map((page, index) => {
      if (
        !page ||
        typeof page !== 'object' ||
        Array.isArray(page) ||
        Object.keys(page).sort().join(',') !== 'locale,outputPath,routeBase' ||
        typeof page.locale !== 'string' ||
        !page.locale ||
        locales.has(page.locale) ||
        typeof page.routeBase !== 'string' ||
        !page.routeBase.endsWith('/') ||
        /[\s?#%]/u.test(page.routeBase) ||
        requestPath(page.routeBase) !== page.routeBase ||
        !page.routeBase.startsWith(basePath) ||
        mounts.has(page.routeBase) ||
        !resourceName(page.outputPath) ||
        outputs.has(page.outputPath.toLowerCase()) ||
        (index === 0 &&
          (page.routeBase !== basePath || page.outputPath !== '404.html'))
      )
        throw invalid();
      locales.add(page.locale);
      mounts.add(page.routeBase);
      outputs.add(page.outputPath.toLowerCase());
      return {
        locale: page.locale,
        routeBase: page.routeBase,
        outputPath: page.outputPath
      };
    });
  };
  const notFound = parseAuxiliary(report.notFound);
  if (
    JSON.stringify(notFound) !==
    JSON.stringify(parseAuxiliary(manifest.notFound))
  )
    throw new Error(
      '[DOCS_PREVIEW_REPORT] auxiliary report and Rust manifest differ'
    );
  const names = new Set<string>();
  const add = (name: unknown) => {
    if (!resourceName(name) || names.has(name))
      throw new Error(
        '[DOCS_PREVIEW_REPORT] invalid or duplicate public output member'
      );
    names.add(name);
  };
  for (const page of report.pages) add(page?.outputPath);
  for (const page of notFound) add(page.outputPath);
  for (const resource of manifest.resources)
    if (resource?.name !== manifest.template) add(resource?.name);
  add('docs-manifest.json');
  add('docs-build-report.json');
  for (const page of notFound) await regularFile(output, page.outputPath);
  return { basePath, names, notFound };
}

/** Static transport only: content identities and the public namespace come from Rust artifacts. */
export async function startDocsPreview(
  config: FarmCliOptions & UserConfig = {},
  execution: DocsPreviewExecution = {}
): Promise<DocsPreviewHandle> {
  const { signal } = execution;
  checkAbort(signal);
  assertNativePlugins(config as DocsFarmConfig);
  const root = await fs.realpath(config.root ?? process.cwd());
  const output = path.resolve(
    root,
    config.server?.preview?.distDir ??
      config.compilation?.output?.path ??
      'build'
  );
  if (
    !inside(root, output) ||
    ['.git', 'node_modules', '.farm-docs', '.farm-docs-locks'].includes(
      path.relative(root, output).split(path.sep)[0].toLowerCase()
    )
  )
    throw new Error(
      '[DOCS_PREVIEW_OUTPUT] output must be a dedicated directory inside the site'
    );
  let directory = root;
  for (const part of path.relative(root, output).split(path.sep)) {
    directory = path.join(directory, part);
    const stat = await fs.lstat(directory);
    if (!stat.isDirectory() || stat.isSymbolicLink())
      throw new Error(
        '[DOCS_PREVIEW_OUTPUT] output directories must not be symlinks'
      );
  }
  const { basePath, names, notFound } = await artifacts(output);
  const auxiliaryOutput = (pathname: string) =>
    [...notFound]
      .filter(
        (page) =>
          pathname === page.routeBase.slice(0, -1) ||
          pathname.startsWith(page.routeBase)
      )
      .sort((left, right) => right.routeBase.length - left.routeBase.length)[0]
      ?.outputPath ?? notFound[0].outputPath;
  checkAbort(signal);
  const resolved = await resolveConfig(
    {
      ...config,
      root,
      configFile: false,
      workflow: undefined,
      plugins: [],
      compilation: {
        ...config.compilation,
        input: { preview: 'farm-docs:preview' },
        output: { ...config.compilation?.output, path: output, publicPath: '/' }
      },
      server: {
        ...config.server,
        appType: 'mpa',
        preview: { ...config.server?.preview, distDir: output }
      }
    },
    'preview',
    'production'
  );
  const server = new PreviewServer({});
  let initialized = false,
    closing: Promise<void> | undefined;
  const close = () => {
    if (!closing) {
      signal?.removeEventListener('abort', abort);
      closing = initialized ? server.close() : Promise.resolve();
    }
    return closing;
  };
  const abort = () => {
    void close().catch((error) => console.error('[DOCS_PREVIEW_CLOSE]', error));
  };
  async function send(
    request: IncomingMessage,
    response: ServerResponse,
    name: string,
    status: number
  ) {
    const { file, stat } = await regularFile(output, name);
    if (!response.hasHeader('Content-Type')) {
      const type = mime.getType(name) ?? 'application/octet-stream';
      response.setHeader(
        'Content-Type',
        /^(?:text\/|application\/(?:javascript|json|xml)|image\/svg\+xml)/.test(
          type
        )
          ? type + '; charset=utf-8'
          : type
      );
    }
    response.setHeader('Content-Length', stat.size);
    response.statusCode = status;
    if (request.method === 'HEAD') response.end();
    else await pipeline(createReadStream(file), response);
  }
  async function serve(request: IncomingMessage, response: ServerResponse) {
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      response.writeHead(405, { Allow: 'GET, HEAD' }).end();
      return;
    }
    const target = request.url ?? '',
      separator = target.indexOf('?');
    const raw = separator < 0 ? target : target.slice(0, separator),
      query = separator < 0 ? '' : target.slice(separator);
    const decoded = requestPath(raw);
    if (decoded === undefined) {
      response.writeHead(400).end();
      return;
    }
    if (
      basePath !== '/' &&
      (decoded === '/' || decoded === basePath.slice(0, -1))
    ) {
      response.writeHead(302, { Location: encodeURI(basePath) + query }).end();
      return;
    }
    if (!decoded.startsWith(basePath)) {
      await send(request, response, auxiliaryOutput(decoded), 404);
      return;
    }
    const relative = decoded.slice(basePath.length);
    const candidates =
      relative.endsWith('/') || !relative
        ? [relative + 'index.html']
        : [relative, relative + '/index.html'];
    for (const name of candidates) {
      if (!names.has(name)) continue;
      try {
        await send(request, response, name, 200);
        return;
      } catch (error) {
        if (response.headersSent) throw error;
        break;
      }
    }
    await send(request, response, auxiliaryOutput(decoded), 404);
  }
  try {
    checkAbort(signal);
    await server.createPreviewServer(resolved, (request, response) => {
      void serve(request, response).catch(() => {
        if (response.destroyed || response.writableEnded) return;
        if (response.headersSent) response.destroy();
        else
          response
            .writeHead(500, { 'Content-Type': 'text/plain; charset=utf-8' })
            .end('Could not read the built site.');
      });
    });
    initialized = true;
    checkAbort(signal);
    await server.listen();
    checkAbort(signal);
    signal?.addEventListener('abort', abort, { once: true });
    const address = server.httpServer.address();
    if (!address || typeof address === 'string')
      throw new Error(
        '[DOCS_PREVIEW_LISTEN] server did not expose a TCP address'
      );
    const host = ['::', '0.0.0.0'].includes(address.address)
      ? 'localhost'
      : address.address.includes(':')
        ? `[${address.address}]`
        : address.address;
    return {
      url: new URL(
        basePath,
        `${server.httpsOptions ? 'https' : 'http'}://${host}:${address.port}`
      ).href,
      close
    };
  } catch (error) {
    await close();
    throw error;
  }
}
