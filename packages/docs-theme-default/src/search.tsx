import { Link } from '@farmfe/docs/client';
import type { ThemeProps } from '@farmfe/docs/theme';
import { useEffect, useId, useRef, useState } from 'react';
import { useSearchQuery } from './search-query.js';

export type SearchProps = Pick<ThemeProps, 'site'>;
function AlgoliaSearch({ site }: SearchProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const overflow = useRef<string | null>(null);
  const [mounted, setMounted] = useState(false);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const { state, retry, invalidate } = useSearchQuery(site, query, open);
  const id = useId();
  function releaseScroll() {
    if (overflow.current !== null) { document.body.style.overflow = overflow.current; overflow.current = null; }
  }
  function dismiss() {
    invalidate();
    dialog.current?.close();
    releaseScroll();
    setOpen(false);
    setQuery('');
  }
  useEffect(() => {
    setMounted(true);
    const navigation = () => { if (dialog.current?.open) dismiss(); };
    window.addEventListener('popstate', navigation);
    window.addEventListener('farm-docs:before-focus', navigation);
    return () => {
      invalidate();
      window.removeEventListener('popstate', navigation);
      window.removeEventListener('farm-docs:before-focus', navigation);
      if (dialog.current?.open) dialog.current.close();
      releaseScroll();
    };
  }, []);
  if (!mounted) return null;
  const results = state.kind === 'ready' ? state.results : [];
  return <>
    <button type="button" aria-haspopup="dialog" aria-controls={id} aria-expanded={open} onClick={() => {
      if (!dialog.current || dialog.current.open) return;
      dialog.current.showModal();
      overflow.current = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      setOpen(true);
      input.current?.focus();
    }}>Search documentation</button>
    <dialog id={id} ref={dialog} className="farm-docs-search" aria-label="Search documentation" onClose={dismiss} onCancel={event => { event.preventDefault(); dismiss(); }} onKeyDown={event => {
      const links = Array.from(event.currentTarget.querySelectorAll<HTMLAnchorElement>('a[href]'));
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        const index = links.indexOf(document.activeElement as HTMLAnchorElement);
        const target = event.key === 'ArrowDown' ? links[index + 1] : index <= 0 ? input.current : links[index - 1];
        if (target) { event.preventDefault(); target.focus(); }
      }
      if (event.key === 'Tab') {
        const stops = Array.from(event.currentTarget.querySelectorAll<HTMLElement>('button,input,a[href]')).filter(element => !element.matches(':disabled') && element.getClientRects().length);
        const first = stops[0], last = stops[stops.length - 1];
        if (first && last && ((!event.shiftKey && document.activeElement === last) || (event.shiftKey && document.activeElement === first))) { event.preventDefault(); (event.shiftKey ? last : first).focus(); }
      }
    }} onClick={event => {
      if (event.target === event.currentTarget) {
        const bounds = event.currentTarget.getBoundingClientRect();
        if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) dismiss();
      }
      if (event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey && event.target instanceof Element && event.target.closest('a[href]')) dismiss();
    }}>
      <button type="button" onClick={dismiss}>Close search</button>
      <label>Search docs<input ref={input} type="search" value={query} onChange={event => { invalidate(); setQuery(event.target.value); }} /></label>
      <p role="status">{state.kind === 'loading' ? 'Searching…' : state.kind === 'error' ? 'Search failed. Try again.' : state.kind === 'ready' && results.length === 0 ? 'No results.' : state.kind === 'ready' ? `${results.length} results.` : 'Type to search.'}</p>
      {state.kind === 'error' && <button type="button" onClick={retry}>Retry search</button>}
      {results.length > 0 && <ul aria-label="Search results">{results.map((result, index) => <li key={`${result.href}:${index}`}><Link to={result.href}>{result.title}</Link>{result.content && <p>{result.content}</p>}</li>)}</ul>}
    </dialog>
  </>;
}

export function Search({ site }: SearchProps) {
  return site.search?.provider === 'algolia' && site.search.options ? <AlgoliaSearch site={site} /> : null;
}
