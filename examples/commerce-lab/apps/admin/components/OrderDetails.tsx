import React from 'react';
import { Badge } from '../../../packages/ui';
import { OrderLineItem } from './OrderLineItem';
import { createFixtures } from '../../../packages/domain/index.mjs';

type State = ReturnType<typeof createFixtures>;
export function OrderDetails({ order, product, onClose }: {
  order: State['orders'][number];
  product: State['products'][number] | undefined;
  onClose: () => void;
}) {
  return (
    <section className="panel" aria-label="Order details">
      <h2>Order {order.id}</h2>
      <dl>
        <dt>Customer</dt><dd>{order.customer}</dd>
        <OrderLineItem order={order} product={product} />
        <dt>Sales channel</dt><dd>{order.channel}</dd>
        <dt>Status</dt><dd><Badge>{order.status}</Badge></dd>
      </dl>
      <button onClick={onClose}>Close order details</button>
    </section>
  );
}
