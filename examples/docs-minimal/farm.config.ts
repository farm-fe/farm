import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'Farm Docs Example',
  url: 'https://example.com',
  basePath: '/guide/',
  docs: { collections: [{ id: 'main', dir: 'docs', routeBase: '/', sidebar: 'sidebar.json' }] },
  theme: { navbar: [{ label: 'Getting started', to: '/guide/getting-started/' }] }
}, {
  compilation: { output: { path: 'build' }, persistentCache: false, sourcemap: false }
});
