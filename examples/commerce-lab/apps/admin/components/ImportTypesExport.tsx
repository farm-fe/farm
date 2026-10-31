import React, { useEffect, useRef, useState } from 'react';
import { Panel } from '../../../packages/ui';
import { ImportTypesPreview } from './ImportTypesPreview';

export function ImportTypesExport({ rows }: { rows?: Record<string, string>[] }) {
  const [source, setSource] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const request = useRef(0);
  useEffect(() => {
    request.current++;
    setSource('');
    setError('');
    setBusy(false);
    return () => { request.current++; };
  }, [rows]);

  const generate = async () => {
    if (!rows?.length) return;
    const id = ++request.current;
    setBusy(true);
    setSource('');
    setError('');
    try {
      const { run } = await import('json_typegen_wasm');
      if (id !== request.current) return;
      const output = run('ProductImport', JSON.stringify({ rows }), JSON.stringify({ output_mode: 'typescript' }));
      if (!output.trim()) throw new Error('No type definitions were generated.');
      setSource(output);
    } catch (error) {
      if (id === request.current) setError(`Could not generate types. Retry the operation. ${String(error)}`);
    } finally {
      if (id === request.current) setBusy(false);
    }
  };

  return <Panel title="3. Share the import contract">
    <p>Generate TypeScript types from all validated CSV rows for your integration team. CSV fields remain strings. These sample-derived types do not replace runtime validation.</p>
    <button type="button" disabled={busy || !rows?.length} onClick={generate}>Generate TypeScript types</button>
    {busy && <p role="status">Generating import types…</p>}
    {!rows?.length && <p>Validate and correct every row before generating types.</p>}
    {error && <p role="alert">{error}</p>}
    {source && <ImportTypesPreview source={source} />}
  </Panel>;
}
