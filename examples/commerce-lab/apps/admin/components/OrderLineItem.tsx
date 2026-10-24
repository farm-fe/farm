import React from 'react';
import { money } from '../../../packages/ui';
import type { Order } from './order-types';
import type { Product } from './product-types';

export function OrderLineItem({ order, product }: { order: Order; product?: Product }) {
  return <>
    <dt>Product</dt><dd>{product?.name ?? 'Product unavailable'}</dd>
    <dt>SKU</dt><dd>{product?.sku ?? order.productId}</dd>
    <dt>Quantity</dt><dd>{order.quantity}</dd>
    <dt>Total charged</dt><dd>{money(order.total)}</dd>
  </>;
}
