import React from 'react';
import { Empty } from '../../../packages/ui';
import { ProductTableRow } from './ProductTableRow';
import type { Product } from './product-types';
export function ProductTable({ products, role, selected, onToggle, onPreview, onEdit, onPublish }: { products: Product[]; role: string; selected: string[]; onToggle: (id: string, checked: boolean) => void; onPreview: (id: string) => void; onEdit: (id: string) => void; onPublish: (id: string) => void }) {
  return <div className="table-wrap"><table aria-label="Product catalog"><thead><tr>{role === 'admin' && <th>Select</th>}<th>Product</th><th>SKU</th><th>Category</th><th>Price</th><th>Stock</th><th>Status</th><th>Actions</th></tr></thead><tbody>{products.map(product => <ProductTableRow key={product.id} product={product} role={role} selected={selected.includes(product.id)} onToggle={onToggle} onPreview={onPreview} onEdit={onEdit} onPublish={onPublish} />)}</tbody></table>{!products.length && <Empty>No products match this search.</Empty>}</div>;
}
