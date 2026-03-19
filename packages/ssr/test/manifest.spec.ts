import { describe, expect, it } from 'vitest';
import {
  injectAssetsIntoHtml,
  resolveAssetsFromManifest,
  type SsrManifest
} from '../src/manifest.js';

describe('ssr manifest helpers', () => {
  it('resolves assets from manifest with entry and modules', () => {
    const manifest: SsrManifest = {
      version: 1,
      entries: {
        '/src/main.ts': {
          js: ['assets/main.js'],
          css: ['assets/main.css'],
          preload: ['assets/vendor.js']
        }
      },
      modules: {
        '/src/pages/About.vue': {
          css: ['assets/about.css']
        }
      }
    };

    const assets = resolveAssetsFromManifest({
      manifest,
      entry: '/src/main.ts',
      usedModuleIds: ['/src/pages/About.vue'],
      publicPath: '/'
    });

    expect(assets.css).toEqual(['/assets/main.css', '/assets/about.css']);
    expect(assets.preload).toEqual(['/assets/vendor.js']);
    expect(assets.scripts).toEqual(['/assets/main.js']);
  });

  it('injects assets into html', () => {
    const html =
      '<html><head><title>t</title></head><body><div id="app"></div></body></html>';
    const injected = injectAssetsIntoHtml({
      html,
      assets: {
        css: ['/assets/main.css'],
        preload: ['/assets/vendor.js'],
        scripts: ['/assets/main.js']
      }
    });

    expect(injected).toContain(
      '<link rel="stylesheet" href="/assets/main.css">'
    );
    expect(injected).toContain(
      '<link rel="modulepreload" href="/assets/vendor.js">'
    );
    expect(injected).toContain(
      '<script type="module" src="/assets/main.js"></script>'
    );
  });
});
