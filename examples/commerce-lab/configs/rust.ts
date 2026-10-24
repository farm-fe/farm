import virtualPlugin from '../../../rust-plugins/virtual/scripts/func.js';
import urlPlugin from '../../../rust-plugins/url/scripts/func.js';
import imagePlugin from '../../../rust-plugins/image/scripts/func.js';
import icons from '../../../rust-plugins/icons/scripts/func.js';
import nativeStrip from '../../../rust-plugins/strip/scripts/func.js';
import modularImport from '../../../rust-plugins/modular-import/scripts/func.js';
import nativeComponents from '../../../rust-plugins/react-components/scripts/func.js';
import react from '@farmfe/plugin-react';
import svgr from '@farmfe/plugin-svgr';
import sass from '@farmfe/plugin-sass';
import tailwind from '@farmfe/plugin-tailwindcss';
import autoImport from '@farmfe/plugin-auto-import';
import yaml from '@farmfe/plugin-yaml';
import dsv from '@farmfe/plugin-dsv';
import mdx from '@farmfe/plugin-mdx';
import worker from '@farmfe/plugin-worker';
import compress from '@farmfe/plugin-compress';
import wasm from '@farmfe/plugin-wasm';
import { channelVirtual } from '../adapters/channel-virtual.mjs';
import { productUrl } from '../adapters/product-url.mjs';
import { productImageDom } from '../adapters/product-image.mjs';
import { commerceIcons } from '../adapters/commerce-icons.mjs';
import { publicationStrip } from '../adapters/publication-strip.mjs';
import { modularUi } from '../adapters/modular-ui.mjs';
import { reactComponents } from '../adapters/react-components.mjs';

export function rustPlugins() {
  return {
    plugins: [
      react({ runtime: 'automatic' }),
      svgr({ exclude: ['product-(home|outdoor|studio|travel)\\.svg'] }),
      sass(),
      tailwind(),
      autoImport({ presets: ['react'], dts: false }),
      yaml(),
      dsv({ include: ['\\.(csv|tsv)$'] }),
      mdx({ include: '\\.mdx$' }),
      worker(),
      compress({ algorithm: 'gzip', level: 6, threshold: 1024, deleteOriginFile: false }),
      wasm(),
      channelVirtual('rust', virtualPlugin),
      productUrl('rust', urlPlugin),
      productImageDom('rust', imagePlugin),
      commerceIcons('rust', icons),
      publicationStrip('rust', undefined, nativeStrip),
      modularUi('rust', modularImport),
      reactComponents('rust', nativeComponents),
    ],
  };
}
