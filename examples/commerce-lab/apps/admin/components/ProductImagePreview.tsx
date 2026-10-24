import React, { useState } from 'react';
import { productImage } from './product-media';
import { money } from '../../../packages/ui';
export function ProductImagePreview({ product, onClose }: { product: { name: string; sku: string; category: string; price: number }; onClose: () => void }) {
  const [zoom, setZoom] = useState(false);
  const [failed, setFailed] = useState(false);
  const [metadata, setMetadata] = useState(false);
  return <section role="dialog" aria-label="Product media" className="panel"><h2>{product.name}</h2>
    <p>{product.sku} · {product.category} · {money(product.price)}</p><p>Category illustration for the local demo catalog.</p>
    {metadata ? <ProductImageMetadata category={product.category} name={product.name} /> : failed ? <p role="alert">Product image could not load.</p> : <div style={{ width: 480, maxWidth: '100%', overflow: 'hidden' }}><img src={productImage(product.category)} alt={`${product.name} category illustration`} width={480} height={360} style={{ maxWidth: '100%', height: 'auto', transform: zoom ? 'scale(1.7)' : 'none' }} onError={() => setFailed(true)} /></div>}
    <button onClick={() => setMetadata(!metadata)}>{metadata ? 'Return to standard preview' : 'Inspect image metadata'}</button>
    <button aria-pressed={zoom} disabled={failed || metadata} onClick={() => setZoom(!zoom)}>Zoom product image</button><button onClick={onClose}>Close product preview</button>
  </section>;
}
