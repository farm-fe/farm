import { defineDocsConfig, type SiteConfig } from '@farmfe/docs';
import { usePage, useSite } from '@farmfe/docs/client';
import { startDocsPreview } from '@farmfe/docs/preview';
import type { NotFoundProps, ThemeProps } from '@farmfe/docs/theme';
export type PublicTypes = {
  site: SiteConfig;
  theme: ThemeProps;
  notFound: NotFoundProps;
};
export const publicEntrypoints = { usePage, useSite, startDocsPreview };
const site: SiteConfig = {
  title: 'Orchard Handbook',
  url: 'https://orchard.example',
  docs: { collections: [{ id: 'guide', dir: 'docs', routeBase: '/' }] }
};
export const publicConfig = defineDocsConfig(site, {
  compilation: { output: { path: 'build' } }
});
