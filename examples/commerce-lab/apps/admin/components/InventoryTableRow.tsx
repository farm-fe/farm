import React from 'react';
import { Badge } from '../../../packages/ui';
import type { Product } from './product-types';
import InventoryIcon from '~icons/commerce/inventory';

export function InventoryTableRow({ product, canReceive, onReceive }: {
  product: Product; canReceive: boolean; onReceive: (id: string) => void;
}) {
  return <tr><td>{product.name}</td><td>{product.sku}</td><td>{product.stock}</td>
    <td><Badge>{product.stock < 25 ? 'low' : 'healthy'}</Badge></td>
    <td>{canReceive && <button aria-label={`Receive stock ${product.sku}`} onClick={() => onReceive(product.id)}><InventoryIcon data-testid="inventory-icon" width={22} height={22} aria-hidden="true" />Receive stock</button>}</td></tr>;
}
