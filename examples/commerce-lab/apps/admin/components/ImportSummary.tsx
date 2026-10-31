import React from 'react';
import { ImportErrors } from './ImportErrors';

export type ImportResult = {
  rows: Record<string, string>[];
  valid: unknown[];
  errors: Array<{ row: number; field: string; message: string }>;
  error?: string;
};

export function ImportSummary({ result, onImport }: { result: ImportResult | null; onImport: () => void }) {
  if (!result) return <p>Run validation to review your collection.</p>;
  if (result.error) return <p role="alert">{result.error}</p>;
  return <><p>{result.valid.length} valid rows · {result.errors.length} errors</p>
    <ImportErrors errors={result.errors} />
    <button className="primary" disabled={!!result.errors.length || !result.rows.length} onClick={onImport}>Import drafts</button>
  </>;
}
