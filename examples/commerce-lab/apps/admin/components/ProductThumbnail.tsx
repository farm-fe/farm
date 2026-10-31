import React from 'react';
import { productImage } from './product-media';
export function ProductThumbnail({ name, sku, category, onPreview }: { name: string; sku: string; category: string; onPreview: () => void }) {
  return <button aria-label={`Preview ${sku}`} onClick={onPreview}><img src={productImage(category)} alt={`${name} category illustration`} width={64} height={48} loading="lazy" />{name}</button>;
}
