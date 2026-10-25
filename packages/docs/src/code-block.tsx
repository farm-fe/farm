import type { HTMLAttributes } from 'react';
import { useEffect, useRef, useState } from 'react';
import { CODE_BLOCK_METADATA_MARKER } from './code-block-metadata.js';

export interface CodeBlockProps extends HTMLAttributes<HTMLPreElement> {
  'data-code'?: string;
  'data-title'?: string;
  'data-language'?: string;
}

/** Display the Rust token tree and copy its original text without reparsing it. */
export function CodeBlock(props: CodeBlockProps) {
  const raw = props['data-code'];
  const title = props['data-title'];
  const language = props['data-language'] ?? 'text';
  const [mounted, setMounted] = useState(false);
  const [status, setStatus] = useState<'idle' | 'pending' | 'copied' | 'failed'>('idle');
  const alive = useRef(false);
  const request = useRef(0);
  useEffect(() => { request.current++; setStatus('idle'); }, [raw]);
  useEffect(() => {
    alive.current = true;
    setMounted(true);
    return () => { alive.current = false; };
  }, []);
  if (typeof raw !== 'string') return <pre {...props} data-farm-docs-code-block={undefined} />;
  const copy = async () => {
    const currentRequest = ++request.current;
    setStatus('pending');
    try {
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(raw);
      if (alive.current && request.current === currentRequest) setStatus('copied');
    } catch {
      if (alive.current && request.current === currentRequest) setStatus('failed');
    }
  };
  return <section className="farm-docs-code" role="group" aria-label={`Code: ${title ?? language}`}>
    <div className="farm-docs-code-header"><span>{title ?? language}</span>
      {mounted && <button className="farm-docs-code-copy" type="button" data-copy-state={status} aria-label={status === 'copied' ? 'Copied code' : status === 'failed' ? 'Could not copy. Try again.' : 'Copy code'} title={status === 'copied' ? 'Copied code' : 'Copy code'} disabled={status === 'pending'} onClick={copy}>
        <svg aria-hidden="true" focusable="false" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          {status === 'copied' ? <path d="m5 12 4 4 10-10" /> : <><rect x="8" y="8" width="12" height="12" rx="2" /><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3" /></>}
        </svg>
      </button>}
    </div>
    <pre {...props} data-farm-docs-code-block={raw.includes('\r') && !raw.includes('\0') ? CODE_BLOCK_METADATA_MARKER : undefined} />
    <span className="farm-docs-code-status" role="status">{status === 'copied' ? 'Copied.' : status === 'failed' ? 'Could not copy. Try again.' : ''}</span>
  </section>;
}
