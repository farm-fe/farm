import React, { useState } from 'react';
import { money } from '../../../packages/ui';

type Product = { id: string; sku: string; name: string; price: number };
export function CampaignProductPicker({ products, selected, onChange }: {
  products: Product[]; selected: string[]; onChange: (ids: string[]) => void;
}) {
  const [search, setSearch] = useState('');
  const visible = products.filter(product => `${product.sku} ${product.name}`.toLowerCase().includes(search.toLowerCase()));
  return <fieldset><legend>Campaign collection · {selected.length} selected</legend>
    <label>Search campaign products<input value={search} onChange={event => setSearch(event.target.value)} /></label>
    <div style={{ maxHeight: 240, overflowY: 'auto' }}>
      {visible.map(product => <label key={product.id}>
        <input type="checkbox" aria-label={`Campaign product ${product.sku}`} checked={selected.includes(product.id)}
          onChange={event => onChange(event.target.checked ? [...selected, product.id] : selected.filter(id => id !== product.id))} />
        {product.name} · {product.sku} · {money(product.price)}
      </label>)}
      {!visible.length && <p>No published products match this search.</p>}
    </div>
  </fieldset>;
}
