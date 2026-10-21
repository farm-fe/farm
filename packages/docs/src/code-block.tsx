import type { HTMLAttributes } from 'react';
import { useEffect, useRef, useState } from 'react';

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
  useEffect(() => {
    alive.current = true;
    setMounted(true);
    return () => { alive.current = false; };
  }, []);
  if (typeof raw !== 'string') return <pre {...props} />;
  const copy = async () => {
    setStatus('pending');
    try {
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(raw);
      if (alive.current) setStatus('copied');
    } catch {
      if (alive.current) setStatus('failed');
    }
  };
  return <section className="farm-docs-code" role="group" aria-label={`Code: ${title ?? language}`}>
    <div className="farm-docs-code-header"><span>{title ?? language}</span>
      {mounted && <button type="button" disabled={status === 'pending'} onClick={copy}>Copy code</button>}
    </div>
    <pre {...props} />
    <span role="status">{status === 'copied' ? 'Copied.' : status === 'failed' ? 'Could not copy. Try again.' : ''}</span>
  </section>;
}
