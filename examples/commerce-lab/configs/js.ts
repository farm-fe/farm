import react from '@farmfe/plugin-react';
import svgr from '@farmfe/js-plugin-svgr';
import sass from '@farmfe/js-plugin-sass';
import tailwind from '@farmfe/js-plugin-tailwindcss';
import worker from '@farmfe/plugin-worker';
import autoImport from 'unplugin-auto-import/vite';
import { data } from '../adapters/data';
import { compression } from '../adapters/compression.mjs';
import { wasm } from '../adapters/wasm.mjs';
import { channelVirtual } from '../adapters/channel-virtual.mjs';
import { productUrl } from '../adapters/product-url.mjs';
import { productImageDom } from '../adapters/product-image.mjs';
import { commerceIcons } from '../adapters/commerce-icons.mjs';
import { publicationStrip } from '../adapters/publication-strip.mjs';
import { modularUi, modularUiJsxPrelude } from '../adapters/modular-ui.mjs';
import { reactComponents } from '../adapters/react-components.mjs';

export function jsPlugins() {
  return {
    plugins: [
      react({ runtime: 'automatic' }),
      svgr({ filters: { resolvedPaths: ['(brand|icon)\\.svg$'] } }),
      sass({ legacy: false }),
      ...tailwind(),
      data(),
      worker(),
      compression(),
      wasm(),
      channelVirtual('js'),
      productUrl('js'),
      productImageDom('js'),
      commerceIcons('js'),
      publicationStrip('js'),
      modularUi('js'),
      modularUiJsxPrelude({ catalog: true }),
      reactComponents('js'),
    ],
    vitePlugins: [autoImport({ imports: ['react'], dts: false })],
  };
}
