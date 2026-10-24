import React from 'react';
import type { Product } from './product-types';
import { InventoryTableRow } from './InventoryTableRow';

export function InventoryTable({ products, canReceive, onReceive }: {
  products: Product[]; canReceive: boolean; onReceive: (id: string) => void;
}) {
  return <div className="table-wrap"><table aria-label="Inventory records">
    <thead><tr><th>Product</th><th>SKU</th><th>Available</th><th>Health</th><th>Action</th></tr></thead>
    <tbody>{products.map(product => <InventoryTableRow key={product.id} product={product} canReceive={canReceive} onReceive={onReceive} />)}</tbody>
  </table></div>;
}
