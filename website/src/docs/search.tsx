import { DocSearch } from '@docsearch/react';
import '@docsearch/css';
import type { SearchProps } from '@farmfe/docs-theme-default';
import { useEffect, useState } from 'react';
/** Website-owned vendor UI. Credentials are the existing public search configuration. */
export function WebsiteSearch({ site }: SearchProps) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const options = site.search?.options;
  if (!mounted || site.search?.provider !== 'algolia' || !options) return null;
  return <DocSearch appId={options.appId} apiKey={options.apiKey} indexName={options.indexName} />;
}
