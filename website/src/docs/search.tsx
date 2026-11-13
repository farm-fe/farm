import { DocSearch } from '@docsearch/react';
import '@docsearch/css';
import type { SearchProps } from '@farmfe/docs-theme-default';
import { createContext, useContext, useEffect, useState } from 'react';
// Keep language selection website-owned; the published theme API is unchanged.
export const WebsiteSearchLocale = createContext('en');

/** Website-owned vendor UI. Only public search-only credentials belong here. */
export function WebsiteSearch({ site }: SearchProps) {
  const locale = useContext(WebsiteSearchLocale);
  const language = locale === 'zh' ? 'zh' : 'en';
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const options = site.search?.options;
  if (!mounted || site.search?.provider !== 'algolia' || !options) return null;
  return <DocSearch key={`${options.indexName}:${language}`} appId={options.appId} apiKey={options.apiKey} indexName={options.indexName} searchParameters={{ facetFilters: [`lang:${language}`] }} />;
}
