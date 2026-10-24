import React from 'react';

export function ImportReviewRow({ row, index, onChange }: {
  row: Record<string, string>;
  index: number;
  onChange: (field: string, value: string) => void;
}) {
  return <tr><td>{index + 2}</td>{['sku', 'name', 'price', 'category'].map(field =>
    <td key={field}><input aria-label={`Row ${index + 2} ${field}`} value={row[field] ?? ''}
      onChange={event => onChange(field, event.target.value)} /></td>)}</tr>;
}
