import React from 'react';
import { Badge, money } from '../../../packages/ui';
import { ProductThumbnail } from './ProductThumbnail';
import { ProductActions } from './ProductActions';
import type { Product } from './product-types';
export function ProductTableRow({ product, role, selected, onToggle, onPreview, onEdit, onPublish }: { product: Product; role: string; selected: boolean; onToggle: (id: string, checked: boolean) => void; onPreview: (id: string) => void; onEdit: (id: string) => void; onPublish: (id: string) => void }) {
  return <tr data-testid="product-row">{role === 'admin' && <td><input type="checkbox" aria-label={`Select ${product.sku}`} checked={selected} onChange={event => onToggle(product.id, event.target.checked)} /></td>}
    <td><ProductThumbnail name={product.name} sku={product.sku} category={product.category} onPreview={() => onPreview(product.id)} /></td><td><code>{product.sku}</code></td><td>{product.category}</td><td>{money(product.price)}</td><td>{product.stock}</td><td><Badge>{product.status}</Badge></td><td><ProductActions product={product} role={role} onEdit={onEdit} onPublish={onPublish} /></td></tr>;
}
