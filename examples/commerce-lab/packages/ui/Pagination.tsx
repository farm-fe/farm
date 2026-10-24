import React from 'react';
export function Pagination({ page, pages, label, onChange }: { page: number; pages: number; label: string; onChange: (page: number) => void }) {
  return <nav aria-label={`${label} pagination`} className="toolbar"><button disabled={page <= 0} onClick={() => onChange(Math.max(0, page - 1))}>Previous {label}</button>
    <span>Page {page + 1} of {pages}</span><button disabled={page + 1 >= pages} onClick={() => onChange(Math.min(pages - 1, page + 1))}>Next {label}</button></nav>;
}
