declare const __COMMERCE_GROUP__: 'js' | 'rust' | 'mixed';
declare const useState: typeof import('react').useState;
declare const ProductImageMetadata: typeof import('./components/ProductImageMetadata').ProductImageMetadata;
declare module '~icons/commerce/*' {
  const component: import('react').ComponentType<import('react').SVGProps<SVGSVGElement>>;
  export default component;
}
declare module 'commerce-channel-capabilities.js' {
  const configuration: { version: string; channels: string[] };
  export default configuration;
}
declare module '*.svg?url' { const url: string; export default url; }
declare module '*.svg?media' { const url: string; export default url; }
declare module '*.svg?dom' { const image: HTMLImageElement; export default image; }
declare module '@commerce-extension' {
  import type { ComponentType } from 'react';
  export const extensionCatalog: Array<{ id: string; title: string; Form: ComponentType }>;
}
declare module '*.svg' {
  import type { ComponentType, SVGProps } from 'react';
  const component: ComponentType<SVGProps<SVGSVGElement>>;
  export default component;
}
declare module '*.csv' {
  const rows: Array<Record<string, string>>;
  export default rows;
}
declare module '*.yaml' {
  const rules: { maxDiscount: number; channels: string[]; reserveStock: number };
  export default rules;
}

declare module '*.tsv' {
  const rows: Array<Record<string, string>>;
  export default rows;
}
declare module '*.mdx' {
  import type { ComponentType } from 'react';
  const page: ComponentType;
  export default page;
}
declare module '*?worker' {
  const constructor: { new (): Worker };
  export default constructor;
}
