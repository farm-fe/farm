declare module 'farm:react-refresh' {
  export function flushRefresh(): Promise<void>;
  export function beginRefreshTransaction(): Promise<import('./development-transaction.js').DevelopmentTransaction & {
    commit(): Promise<void>;
    rollback(): Promise<unknown[]>;
  }>;
}
declare module 'docs:application' {
  import type { ReactElement } from 'react';
  import type { PublicSiteConfig, PublicPage, PublicSeoHead } from './config.generated.js';
  export const site: PublicSiteConfig;
  export const home: string;
  export const pages: PublicPage[];
  export const routeAliases: Record<string, string>;
  export const seoByKey: Record<string, PublicSeoHead>;
  export const notFoundPages: import('./theme.js').PublicNotFound[];
  export function getNotFound(path: string): import('./theme.js').PublicNotFound;
  export const snapshotId: string;
  export function loadPage(key: string): Promise<void>;
  export function renderPage(key: string): ReactElement;
  export function renderNotFound(locale?: string): ReactElement;
}
