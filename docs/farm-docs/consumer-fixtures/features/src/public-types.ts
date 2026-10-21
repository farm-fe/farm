import { defineDocsConfig, type SiteConfig } from '@farmfe/docs';
import sass from '@farmfe/plugin-sass';
import tailwindcss from '@farmfe/plugin-tailwindcss';

const site: SiteConfig = {
  title: 'Orchard Field Notes',
  url: 'https://orchard.example',
  basePath: '/manual/',
  docs: {
    collections: [
      { id: 'guide', dir: 'content/en-current', routeBase: '/docs/' }
    ]
  }
};
export const publicConfig = defineDocsConfig(site, {
  plugins: [
    sass(),
    tailwindcss({ config: { corePlugins: { preflight: false } } })
  ],
  compilation: { output: { path: 'build' } }
});
