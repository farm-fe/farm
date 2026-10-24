import React from 'react';
import { Stat, money } from '../../../packages/ui';
import type { Customer } from './CustomerList';

export function CustomerPurchaseSummary({ customer }: { customer: Customer }) {
  return <>
    <p>Net spend <strong data-testid="customer-net-spend">{money(customer.revenue)}</strong></p>
    <div className="stats">
      <Stat label="Orders" value={customer.orders} detail="Includes refunded order history" />
      <Stat label="Refunded orders" value={customer.refundedOrders} detail={money(customer.refundedAmount)} />
      <Stat label="Last purchase day" value={customer.lastDay} detail={customer.channels.join(', ')} />
    </div>
  </>;
}
