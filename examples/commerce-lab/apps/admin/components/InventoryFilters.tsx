import React from 'react';

export function InventoryFilters({ search, lowOnly, onSearch, onLowOnly }: {
  search: string; lowOnly: boolean;
  onSearch: (value: string) => void; onLowOnly: (value: boolean) => void;
}) {
  return <div className="toolbar"><input aria-label="Search inventory" value={search} onChange={event => onSearch(event.target.value)} />
    <label><input type="checkbox" checked={lowOnly} onChange={event => onLowOnly(event.target.checked)} />Low stock only</label></div>;
}
