import { fileURLToPath } from 'node:url';
import { defineDocsConfig, type SiteConfig } from '@farmfe/docs';
import sass from '@farmfe/plugin-sass';
import tailwindcss from '@farmfe/plugin-tailwindcss';
import tailwindOptions from './farm.tailwind.json';
import site from './farm-docs.site.json';

// Site migration paths are configuration, not a framework compatibility layer.
export default defineDocsConfig(site as SiteConfig, {
  snippets: {
    rendererModule: fileURLToPath(
      new URL('./src/docs/snippets.tsx', import.meta.url)
    )
  },
  plugins: [sass(), tailwindcss(tailwindOptions)],
  compilation: {
    resolve: {
      alias: { '@site': fileURLToPath(new URL('.', import.meta.url)) }
    },
    output: { path: 'build-farm' },
    persistentCache: false,
    sourcemap: false
  }
});
