import { useEffect, useRef, useState } from 'react';
import { isDocumentActionUrl, readVerifiedMarkdown } from './document-actions-data.js';

export interface DocumentActionsLabels {
  region: string;
  copy: string;
  copying: string;
  copied: string;
  copyFailed: string;
  view: string;
  guide: string;
}

export interface DocumentActionsProps {
  /** The native public page's Markdown URL and exact byte identity. */
  markdownUrl?: string | null;
  markdownSha256?: string | null;
  agentGuideUrl?: string | null;
  labels?: Partial<DocumentActionsLabels>;
}

const defaultLabels: DocumentActionsLabels = {
  region: 'Document formats',
  copy: 'Copy Markdown',
  copying: 'Copying…',
  copied: 'Markdown copied',
  copyFailed: 'Could not copy Markdown. Open the Markdown link instead.',
  view: 'View Markdown',
  guide: 'Agent guide'
};

/** No provider, installation claim or chat service is implied by these actions. */
export function DocumentActions({ markdownUrl, markdownSha256, agentGuideUrl, labels: overrides }: DocumentActionsProps) {
  const labels = { ...defaultLabels, ...overrides };
  const [state, setState] = useState<'idle' | 'copying' | 'copied' | 'failed'>('idle');
  const lifetime = useRef(0);
  useEffect(() => {
    setState('idle');
    return () => { lifetime.current++; };
  }, [markdownUrl, markdownSha256]);
  const markdown = isDocumentActionUrl(markdownUrl) && typeof markdownSha256 === 'string' && /^[a-f0-9]{64}$/.test(markdownSha256);
  const guide = isDocumentActionUrl(agentGuideUrl);
  if (!markdown && !guide) return null;
  return <nav className="farm-docs-document-actions" aria-label={labels.region}>
    {markdown && <>
      <button type="button" disabled={state === 'copying'} onClick={async () => {
        const request = ++lifetime.current;
        setState('copying');
        try {
          const text = await readVerifiedMarkdown(markdownUrl, markdownSha256);
          if (request !== lifetime.current) return;
          await navigator.clipboard.writeText(text);
          if (request === lifetime.current) setState('copied');
        } catch {
          if (request === lifetime.current) setState('failed');
        }
      }}>{state === 'copying' ? labels.copying : labels.copy}</button>
      <a href={markdownUrl} type="text/markdown">{labels.view}</a>
    </>}
    {guide && <a href={agentGuideUrl}>{labels.guide}</a>}
    <span role="status" aria-live="polite">{state === 'copied' ? labels.copied : state === 'failed' ? labels.copyFailed : ''}</span>
  </nav>;
}
