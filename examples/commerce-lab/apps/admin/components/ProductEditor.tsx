import React, { useState } from 'react';
import { productFields } from '../../../packages/domain/product-fields.mjs';
import type { Product } from './product-types';
import { ProductFieldsEditor } from './ProductFieldsEditor';
export function ProductEditor({ product, onSave, onCancel }: { product: Product; onSave: (fields: { name: string; price: number; stock: number }) => void; onCancel: () => void }) {
  const [name, setName] = useState(product.name); const [price, setPrice] = useState((product.price / 100).toFixed(2)); const [stock, setStock] = useState(String(product.stock)); const [error, setError] = useState('');
  return <form className="panel" aria-label="Product editor" onSubmit={event => { event.preventDefault(); try { onSave(productFields(name, price, stock)); setError(''); } catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); } }}>
    <h2>Edit product</h2><ProductFieldsEditor name={name} price={price} stock={stock} onName={setName} onPrice={setPrice} onStock={setStock} />
    {error && <p role="alert">{error}</p>}<button type="submit">Save product</button><button type="button" onClick={onCancel}>Cancel</button></form>;
}
