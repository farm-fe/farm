import type { ComponentType, ReactNode } from 'react';
import type {
  NotFoundPlan,
  PublicBlogIndex,
  PublicPage,
  PublicSidebar,
  PublicSiteConfig
} from './config.generated.js';

export type { PublicSidebarItem } from './config.generated.js';
export interface LayoutProps {
  site: PublicSiteConfig;
  page: PublicPage;
  children: ReactNode;
}
export interface ThemeData {
  home: string;
  pages: PublicPage[];
  sidebars: PublicSidebar[];
  blogIndexes: PublicBlogIndex[];
}
export interface ThemeProps extends LayoutProps {
  data: ThemeData;
  components: ThemeComponents;
  /** Original Rust-selected article components, already loaded for full listings. */
  listingPosts?: Array<{ page: PublicPage; children: ReactNode }>;
}
export type PublicNotFound = Omit<NotFoundPlan, 'outputPath' | 'headHtml'> & {
  robots: string;
};
export interface NotFoundProps {
  site: PublicSiteConfig;
  data: ThemeData & { notFound: PublicNotFound };
  components: ThemeComponents;
}
export interface ThemeComponents {
  Layout: ComponentType<ThemeProps>;
  DocPage: ComponentType<ThemeProps>;
  BlogPage: ComponentType<ThemeProps>;
  Navbar: ComponentType<ThemeProps>;
  Sidebar: ComponentType<ThemeProps>;
  Toc: ComponentType<ThemeProps>;
  Footer: ComponentType<ThemeProps>;
  NotFound: ComponentType<NotFoundProps>;
}
export interface ThemeModule extends Partial<ThemeComponents> {
  default?: ComponentType<ThemeProps>;
  components?: Record<string, ComponentType>;
}

/** Compose explicit React slots; this never discovers routes or content. */
export function resolveTheme(
  defaults: ThemeModule,
  overrides: ThemeModule = {}
) {
  const components: Partial<Record<keyof ThemeComponents, unknown>> = {};
  for (const name of [
    'Layout',
    'DocPage',
    'BlogPage',
    'Navbar',
    'Sidebar',
    'Toc',
    'Footer',
    'NotFound'
  ] as const) {
    const override =
      overrides[name] !== undefined
        ? overrides[name]
        : name === 'Layout'
          ? overrides.default
          : undefined;
    const value = override === undefined ? defaults[name] : override;
    const marker =
      typeof value === 'object' && value
        ? (value as { $$typeof?: symbol }).$$typeof
        : undefined;
    if (
      typeof value !== 'function' &&
      marker !== Symbol.for('react.memo') &&
      marker !== Symbol.for('react.forward_ref') &&
      marker !== Symbol.for('react.lazy')
    )
      throw new Error(`[DOCS_THEME] ${name} must be a React component`);
    components[name] = value;
  }
  for (const module of [defaults, overrides]) {
    if (
      module.components !== undefined &&
      (!module.components ||
        typeof module.components !== 'object' ||
        Array.isArray(module.components))
    )
      throw new Error(
        '[DOCS_THEME] components must be a content component map'
      );
  }
  return {
    components: components as unknown as ThemeComponents,
    contentComponents: { ...defaults.components, ...overrides.components }
  };
}
