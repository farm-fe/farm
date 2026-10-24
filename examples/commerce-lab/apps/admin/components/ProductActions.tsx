import React from 'react';
import type { Product } from './product-types';
export function ProductActions({ product, role, onEdit, onPublish }: { product: Product; role: string; onEdit: (id: string) => void; onPublish: (id: string) => void }) {
  return role === 'admin' ? <><button onClick={() => onEdit(product.id)}>Edit</button>{product.status === 'draft' && <button onClick={() => onPublish(product.id)}>Publish</button>}</> : null;
}
