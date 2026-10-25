import { Link, navigate } from '@farmfe/docs/client';
import { useEffect, useState } from 'react';
import type { SearchProps } from './search.js';
import { useSearchQuery } from './search-query.js';

export interface SearchPageLabels {
  input: string;
  submit: string;
  idle: string;
  loading: string;
  error: string;
  empty: string;
  retry: string;
  results: string;
  unconfigured: string;
  count: (count: number) => string;
}
export interface SearchPageProps extends SearchProps { labels?: Partial<SearchPageLabels> }
const defaults: SearchPageLabels = {
  input: 'Search docs', submit: 'Search', idle: 'Type to search.', loading: 'Searching…',
  error: 'Search failed. Try again.', empty: 'No results.', retry: 'Retry search',
  results: 'Search results', unconfigured: 'Search is not configured.', count: count => `${count} results.`
};

/** A normal content page; its route, locale and metadata come from the Rust page index. */
export function SearchPage({ site, labels }: SearchPageProps) {
  const text = { ...defaults, ...labels };
  const [mounted, setMounted] = useState(false);
  const [query, setQuery] = useState('');
  const [draft, setDraft] = useState('');
  const configured = site.search?.provider === 'algolia' && Boolean(site.search.options);
  const { state, retry, invalidate } = useSearchQuery(site, query, mounted && configured);
  useEffect(() => {
    const synchronize = () => {
      const current = new URL(window.location.href).searchParams.get('q') ?? '';
      setQuery(current); setDraft(current); retry();
    };
    setMounted(true);
    synchronize();
    window.addEventListener('farm-docs:before-focus', synchronize);
    return () => window.removeEventListener('farm-docs:before-focus', synchronize);
  }, []);
  if (!configured) return <p className="farm-docs-search-page">{text.unconfigured}</p>;
  const results = state.kind === 'ready' ? state.results : [];
  return <section className="farm-docs-search-page">
    <form role="search" method="get" onSubmit={event => {
      event.preventDefault();
      const url = new URL(window.location.href);
      if (draft.trim()) url.searchParams.set('q', draft.trim()); else url.searchParams.delete('q');
      invalidate();
      navigate(url.href);
    }}>
      <label>{text.input}<input name="q" type="search" value={draft} onChange={event => setDraft(event.target.value)} /></label>
      <button type="submit">{text.submit}</button>
    </form>
    <p role="status">{state.kind === 'loading' ? text.loading : state.kind === 'error' ? text.error : state.kind === 'ready' && results.length === 0 ? text.empty : state.kind === 'ready' ? text.count(results.length) : text.idle}</p>
    {state.kind === 'error' && <button type="button" onClick={retry}>{text.retry}</button>}
    {results.length > 0 && <ul aria-label={text.results}>{results.map((result, index) => <li key={`${result.href}:${index}`}><Link to={result.href}>{result.title}</Link>{result.content && <p>{result.content}</p>}</li>)}</ul>}
  </section>;
}
