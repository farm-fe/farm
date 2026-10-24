import React, { useState } from 'react';
import { useStore } from '../store';
import { queryProducts, updateProduct, publishProducts } from '../../../packages/domain/index.mjs';
import { filterProducts } from '../../../packages/domain/product-query.mjs';
import { PageTitle } from '../../../packages/ui';
import { ProductImagePreview } from '../components/ProductImagePreview';
import { ProductFilters } from '../components/ProductFilters';
import { ProductBulkActions } from '../components/ProductBulkActions';
import { ProductTable } from '../components/ProductTable';
import { ProductEditor } from '../components/ProductEditor';
export default function Products() {
  const { state, update, tenant, role } = useStore();
  const [filters, setFilters] = useState({ search: '', category: 'all', status: 'all' });
  const [editing, setEditing] = useState<string | null>(null);
  const [error, setError] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const [preview, setPreview] = useState<string | null>(null);
  const owned = queryProducts(state, tenant);
  const categories = [...new Set(owned.map(product => product.category))].sort();
  const previewProduct = owned.find(product => product.id === preview);
  const editingProduct = owned.find(product => product.id === editing);
  const products = filterProducts(state, tenant, filters);
  return <><PageTitle eyebrow="Catalog / Products" title="Your next bestsellers">Curate the collection. Publish when it is ready.</PageTitle>
    <ProductFilters value={filters} categories={categories} onChange={setFilters} />
    <ProductBulkActions selected={selected.length} role={role} onClear={() => setSelected([])} onPublish={() => {
      try { update(publishProducts(state, tenant, selected)); setSelected([]); setError(''); }
      catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); }
    }} />
    {error && <p role="alert">{error}</p>}
    <ProductTable products={products} role={role} selected={selected} onToggle={(id, checked) => setSelected(checked ? [...new Set([...selected, id])] : selected.filter(value => value !== id))} onPreview={setPreview} onEdit={id => { setEditing(id); setError(''); }} onPublish={id => {
      try { update(updateProduct(state, tenant, id, { status: 'published' })); setError(''); }
      catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); }
    }} />
    {role === 'admin' && editingProduct && <ProductEditor key={editingProduct.id} product={editingProduct} onCancel={() => setEditing(null)} onSave={fields => { update(updateProduct(state, tenant, editingProduct.id, fields)); setEditing(null); setError(''); }} />}
    {previewProduct && <ProductImagePreview key={previewProduct.id} product={previewProduct} onClose={() => setPreview(null)} />}
  </>;
}
