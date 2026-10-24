import React, { useState, useRef, useEffect } from 'react';
import Papa from 'papaparse';
import ImportWorker from '../workers/import?worker';
import template from '../content/template.csv';
import { useStore } from '../store';
import {
  queryProducts,
  importProducts,
} from '../../../packages/domain/index.mjs';
import { PageTitle, Panel } from '../../../packages/ui';
import { ImportReview } from '../components/ImportReview';

import { ImportSource } from '../components/ImportSource';
import { ImportValidationActions } from '../components/ImportValidationActions';
import { ImportSummary, type ImportResult } from '../components/ImportSummary';
import { ImportTypesExport } from '../components/ImportTypesExport';

export default function ImportPage() {
  const { state, update, tenant } = useStore();
  const [csv, setCsv] = useState(
    'sku,name,price,category\nNEW-100,Weekend tote,39.50,Travel',
  );
  const [result, setResult] = useState<ImportResult | null>(null);
  const [reviewRows, setReviewRows] = useState<Record<string, string>[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const active = useRef<Worker | null>(null);
  const generation = useRef(0);
  useEffect(
    () => () => {
      generation.current++;
      active.current?.terminate();
    },
    [],
  );
  useEffect(() => {
    generation.current++;
    active.current?.terminate();
    setBusy(false);
    setResult(null);
    setMessage('');
    setReviewRows(null);
  }, [tenant]);
  const invalidate = () => {
    generation.current++;
    active.current?.terminate();
    setBusy(false);
    setResult(null);
    setMessage('');
  };
  const validate = () => {
    active.current?.terminate();
    const requestId = ++generation.current;
    setBusy(true);
    setResult(null);
    setMessage('');
    setReviewRows(null);
    let worker: Worker;
    try {
      worker = new ImportWorker();
    } catch {
      setMessage('Worker failed. Please retry.');
      setBusy(false);
      return;
    }
    active.current = worker;
    worker.onmessage = ({ data }) => {
      if (data.requestId !== generation.current) return;
      setResult(data);
      setReviewRows(data.rows ?? null);
      setBusy(false);
      worker.terminate();
    };
    worker.onerror = () => {
      if (requestId !== generation.current) return;
      setMessage('Worker failed. Please retry.');
      setBusy(false);
      worker.terminate();
    };
    try {
      worker.postMessage({ requestId, csv, existing: queryProducts(state, tenant) });
    } catch {
      setMessage('Worker failed. Please retry.');
      setBusy(false);
      worker.terminate();
    }
  };
  const download = () => {
    const url = URL.createObjectURL(
      new Blob([Papa.unparse(template)], { type: 'text/csv' }),
    );
    const a = document.createElement('a');
    a.href = url;
    a.download = 'product-template.csv';
    a.click();
    URL.revokeObjectURL(url);
  };
  return (
    <>
      <PageTitle
        eyebrow="Catalog / Import"
        title="Bring your collection into focus"
      >
        Validate every row before changing your catalog.
      </PageTitle>
      <div className="grid-two">
        <Panel title="1. Prepare your file">
          <ImportSource csv={csv} onDownload={download} onUpload={async file => {
            invalidate();
            setReviewRows(null);
            const requestId = generation.current;
            setCsv('');
            setBusy(true);
            try {
              const content = await file.text();
              if (requestId === generation.current) setCsv(content);
            } catch {
              if (requestId === generation.current) setMessage('Could not read CSV. Please select the file again.');
            } finally {
              if (requestId === generation.current) setBusy(false);
            }
          }} onChange={content => {
            invalidate();
            setReviewRows(null);
            setCsv(content);
          }} />
          <ImportValidationActions busy={busy} onValidate={validate} onCancel={invalidate} />
        </Panel>
        <Panel title="2. Review and import">
          <ImportSummary result={result} onImport={() => {
            if (!result) return;
            try {
              update(importProducts(state, tenant, result.rows));
              setMessage(result.rows.length + ' draft products imported.');
              setResult(null);
              setReviewRows(null);
            } catch (error) {
              setMessage(String(error));
            }
          }} />
          {reviewRows && <ImportReview rows={reviewRows} onChange={rows => {
            invalidate();
            setReviewRows(rows);
            setCsv(Papa.unparse(rows));
          }} />}
          {message && <p role="status">{message}</p>}
        </Panel>
      </div>
      <ImportTypesExport key={tenant} rows={result && !result.error && !result.errors.length ? result.rows : undefined} />
    </>
  );
}
