import React from 'react';
import type { ProductFiltersValue } from './product-types';
export function ProductFilters({ value, categories, onChange }: { value: ProductFiltersValue; categories: string[]; onChange: (value: ProductFiltersValue) => void }) {
  return <div className="toolbar"><input aria-label="Search products" placeholder="Search name, SKU or category" value={value.search} onChange={event => onChange({ ...value, search: event.target.value })} />
    <select aria-label="Product category" value={value.category} onChange={event => onChange({ ...value, category: event.target.value })}><option value="all">All categories</option>{categories.map(category => <option key={category}>{category}</option>)}</select>
    <select aria-label="Product status" value={value.status} onChange={event => onChange({ ...value, status: event.target.value })}><option value="all">All publication states</option><option value="draft">draft</option><option value="published">published</option></select></div>;
}
