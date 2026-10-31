import React, { useState } from 'react';
import { Pagination } from '../../../packages/ui/Pagination';
import { ImportReviewRow } from './ImportReviewRow';

export function ImportReview({ rows, onChange }: {
  rows: Record<string, string>[];
  onChange: (rows: Record<string, string>[]) => void;
}) {
  const fields = ['sku', 'name', 'price', 'category'];
  const [page, setPage] = useState(0);
  const pages = Math.max(1, Math.ceil(rows.length / 10));
  const currentPage = Math.min(page, pages - 1);
  const start = currentPage * 10;
  return (
    <div className="table-wrap">
      <table aria-label="Import row corrections">
        <thead><tr><th>CSV row</th>{fields.map(field => <th key={field}>{field}</th>)}</tr></thead>
        <tbody>{rows.slice(start, start + 10).map((row, offset) => <ImportReviewRow key={start + offset}
          row={row} index={start + offset} onChange={(field, value) => {
            onChange(rows.map((original, index) => index === start + offset ? { ...original, [field]: value } : original));
          }} />)}</tbody>
      </table>
      <Pagination page={currentPage} pages={pages} label="import rows" onChange={setPage} />
    </div>
  );
}
