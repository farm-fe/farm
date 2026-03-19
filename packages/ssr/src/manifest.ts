import type { SsrRuntimeAssets } from './runtime-types.js';

export type SsrManifestEntry = {
  js?: string[];
  css?: string[];
  preload?: string[];
};

export type SsrManifest = {
  version: 1;
  entries: Record<string, SsrManifestEntry>;
  modules: Record<string, SsrManifestEntry>;
  assets?: Record<string, { type: string }>;
};

export type SsrBuildInfo = {
  version: 1;
  client: {
    outputDir: string;
    manifest: string;
    entry?: string;
  };
  server: {
    outputDir: string;
    entry: string;
  };
};

type ResolveAssetsParams = {
  manifest: SsrManifest | null;
  entry?: string | null;
  usedModuleIds?: string[];
  publicPath?: string;
};

function normalizePublicPath(publicPath: string | undefined): string {
  if (!publicPath) {
    return '/';
  }

  if (publicPath.endsWith('/')) {
    return publicPath;
  }

  return `${publicPath}/`;
}

function normalizeAssetPath(publicPath: string, asset: string): string {
  const cleanAsset = asset.replace(/^\/+/, '');
  return `${publicPath}${cleanAsset}`;
}

function pushUnique(target: string[], source?: string[]) {
  if (!source || source.length === 0) {
    return;
  }
  for (const item of source) {
    if (!target.includes(item)) {
      target.push(item);
    }
  }
}

export function resolveAssetsFromManifest(
  params: ResolveAssetsParams
): SsrRuntimeAssets {
  const assets: SsrRuntimeAssets = {
    css: [],
    preload: [],
    scripts: []
  };

  if (!params.manifest) {
    return assets;
  }

  const publicPath = normalizePublicPath(params.publicPath);
  const entryKey = params.entry ?? null;
  const entryAssets = entryKey ? params.manifest.entries[entryKey] : null;

  if (entryAssets) {
    pushUnique(
      assets.css,
      entryAssets.css?.map((item) => normalizeAssetPath(publicPath, item))
    );
    pushUnique(
      assets.preload,
      entryAssets.preload?.map((item) => normalizeAssetPath(publicPath, item))
    );
    pushUnique(
      assets.scripts,
      entryAssets.js?.map((item) => normalizeAssetPath(publicPath, item))
    );
  }

  for (const moduleId of params.usedModuleIds ?? []) {
    const moduleAssets = params.manifest.modules[moduleId];
    if (!moduleAssets) {
      continue;
    }

    pushUnique(
      assets.css,
      moduleAssets.css?.map((item) => normalizeAssetPath(publicPath, item))
    );
    pushUnique(
      assets.preload,
      moduleAssets.preload?.map((item) => normalizeAssetPath(publicPath, item))
    );
    pushUnique(
      assets.scripts,
      moduleAssets.js?.map((item) => normalizeAssetPath(publicPath, item))
    );
  }

  return assets;
}

export function injectAssetsIntoHtml(params: {
  html: string;
  assets: SsrRuntimeAssets;
}): string {
  const headLinks: string[] = [];
  const bodyScripts: string[] = [];

  for (const href of params.assets.css) {
    headLinks.push(`<link rel="stylesheet" href="${href}">`);
  }

  for (const href of params.assets.preload) {
    headLinks.push(`<link rel="modulepreload" href="${href}">`);
  }

  for (const src of params.assets.scripts) {
    bodyScripts.push(`<script type="module" src="${src}"></script>`);
  }

  let html = params.html;
  if (headLinks.length) {
    const injection = `\n${headLinks.join('\n')}\n`;
    if (html.includes('</head>')) {
      html = html.replace('</head>', `${injection}</head>`);
    } else {
      html = injection + html;
    }
  }

  if (bodyScripts.length) {
    const injection = `\n${bodyScripts.join('\n')}\n`;
    if (html.includes('</body>')) {
      html = html.replace('</body>', `${injection}</body>`);
    } else {
      html = html + injection;
    }
  }

  return html;
}
