import React from 'react';
import type { CampaignQuote } from '../../../packages/sdk/src';
import { money } from '../../../packages/ui';
type Product = { id: string; name: string; sku: string };
export function CampaignPriceTable({ items, products }: { items: CampaignQuote['items']; products: Product[] }) {
  return <table aria-label="Campaign preview prices"><thead><tr><th>Product</th><th>Original</th><th>Offer</th><th>Available</th></tr></thead><tbody>{items.map(item => {
    const product = products.find(product => product.id === item.productId);
    return <tr key={item.productId}><td>{product?.name ?? item.productId} · {product?.sku}</td><td>{money(item.originalPrice)}</td><td>{money(item.salePrice)}</td><td>{item.availableStock}</td></tr>;
  })}</tbody></table>;
}
