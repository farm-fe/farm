import type { ThemeProps } from '@farmfe/docs/theme';
import { useCallback, useEffect, useRef, useState } from 'react';

export interface SearchResult {
  title: string;
  content?: string;
  href: string;
}
export type SearchQueryState =
  | { kind: 'idle' | 'loading' | 'error' }
  | { kind: 'ready'; results: SearchResult[] };

function resultsFrom(value: unknown, siteUrl: string): SearchResult[] {
  if (
    !value ||
    typeof value !== 'object' ||
    !('hits' in value) ||
    !Array.isArray(value.hits)
  )
    throw new Error('Invalid search response');
  return value.hits.slice(0, 20).flatMap((hit: unknown) => {
    if (
      !hit ||
      typeof hit !== 'object' ||
      !('url' in hit) ||
      typeof hit.url !== 'string'
    )
      return [];
    const record = hit as Record<string, unknown>;
    let url: URL;
    try {
      url = new URL(hit.url, siteUrl);
    } catch {
      return [];
    }
    if (
      !['http:', 'https:'].includes(url.protocol) ||
      url.username ||
      url.password
    )
      return [];
    const hierarchy =
      record.hierarchy && typeof record.hierarchy === 'object'
        ? (record.hierarchy as Record<string, unknown>)
        : {};
    const title = [
      record.title,
      ...[6, 5, 4, 3, 2, 1, 0].map((level) => hierarchy[`lvl${level}`]),
      record.content
    ].find((item) => typeof item === 'string' && item.trim()) as
      | string
      | undefined;
    const href =
      url.origin === new URL(siteUrl).origin
        ? url.pathname + url.search + url.hash
        : url.href;
    return [
      {
        title: title ?? url.pathname,
        content:
          typeof record.content === 'string' ? record.content : undefined,
        href
      }
    ];
  });
}

/** Browser-only query transport; content compilation and routes remain Rust-owned. */
export function useSearchQuery(
  site: ThemeProps['site'],
  query: string,
  enabled: boolean
) {
  const request = useRef<AbortController | null>(null);
  const generation = useRef(0);
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<SearchQueryState>({ kind: 'idle' });
  const options =
    site.search?.provider === 'algolia' ? site.search.options : undefined;
  const appId = options?.appId,
    apiKey = options?.apiKey,
    indexName = options?.indexName;
  const invalidate = useCallback(() => {
    generation.current++;
    request.current?.abort();
    request.current = null;
    setState({ kind: 'idle' });
  }, []);
  const retry = useCallback(() => {
    invalidate();
    setAttempt((value) => value + 1);
  }, [invalidate]);

  useEffect(() => {
    const text = query.trim();
    if (
      !enabled ||
      !text ||
      appId === undefined ||
      apiKey === undefined ||
      indexName === undefined
    ) {
      setState({ kind: 'idle' });
      return;
    }
    const controller = new AbortController();
    request.current = controller;
    const owner = ++generation.current;
    let timeout: ReturnType<typeof setTimeout> | undefined;
    setState({ kind: 'loading' });
    const debounce = setTimeout(async () => {
      timeout = setTimeout(() => controller.abort(), 10000);
      try {
        // Restrict the application ID to a hostname label, never a caller URL.
        if (!/^[a-z0-9]+$/i.test(appId))
          throw new Error('Invalid search application');
        const response = await fetch(
          `https://${appId.toLowerCase()}.algolia.net/1/indexes/${encodeURIComponent(indexName)}/query`,
          {
            method: 'POST',
            signal: controller.signal,
            headers: {
              'content-type': 'application/json',
              'x-algolia-application-id': appId,
              'x-algolia-api-key': apiKey
            },
            body: JSON.stringify({
              params: new URLSearchParams({
                query: text,
                hitsPerPage: '20'
              }).toString()
            })
          }
        );
        if (!response.ok) throw new Error('Search unavailable');
        const results = resultsFrom(await response.json(), site.url);
        if (owner === generation.current) setState({ kind: 'ready', results });
      } catch {
        if (owner === generation.current) setState({ kind: 'error' });
      } finally {
        clearTimeout(timeout);
      }
    }, 150);
    return () => {
      generation.current++;
      clearTimeout(debounce);
      clearTimeout(timeout);
      controller.abort();
      if (request.current === controller) request.current = null;
    };
  }, [enabled, query, attempt, appId, apiKey, indexName, site.url]);

  return { state, retry, invalidate };
}
