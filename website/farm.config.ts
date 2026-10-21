import { fileURLToPath } from 'node:url';
import { defineDocsConfig, type SiteConfig } from '@farmfe/docs';
import sass from '@farmfe/plugin-sass';
import tailwindcss from '@farmfe/plugin-tailwindcss';
import tailwindOptions from './farm.tailwind.json';
import site from './farm-docs.site.json';

// Rust validates this independent configuration; legacy site config is not loaded.
export default defineDocsConfig(site as SiteConfig, {
  snippets: {
    rendererModule: fileURLToPath(
      new URL('./src/docs/snippets.tsx', import.meta.url)
    )
  },
  plugins: [sass(), tailwindcss(tailwindOptions)],
  compilation: {
    output: { path: 'build-farm' },
    persistentCache: false,
    sourcemap: false
  }
});
