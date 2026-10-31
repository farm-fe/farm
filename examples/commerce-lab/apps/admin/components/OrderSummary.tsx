import React from 'react';
import { Stat, money } from '../../../packages/ui';
import { orderSummary } from '../../../packages/domain/order-query.mjs';
import type { Order } from './order-types';
export function OrderSummary({ orders }: { orders: Order[] }) {
  const total = orderSummary(orders);
  return <div className="stats" data-testid="order-summary"><Stat label="Matching orders" value={total.orders} detail="Current search and filters" /><Stat label="Net charged" value={money(total.netRevenue)} detail="Excludes refunded orders" /><Stat label="Refunded amount" value={money(total.refundedAmount)} detail="Current filtered orders" /><Stat label="Ordered units" value={total.units} detail="Includes refunded orders" /></div>;
}
