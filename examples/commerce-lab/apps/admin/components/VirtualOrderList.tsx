import React, { useState } from 'react';
import { virtualWindow } from '../../../packages/ui/virtual-window.mjs';
import { OrderTableRow } from './OrderTableRow';
import type { Order } from './order-types';
export function VirtualOrderList({ orders, role, onSelect, onTransition }: { orders: Order[]; role: string; onSelect: (id: string) => void; onTransition: (id: string, status: string) => void }) {
  const [scrollTop, setScrollTop] = useState(0);
  const window = virtualWindow(orders.length, scrollTop, 480, 48);
  return <><div className="table-wrap" data-testid="order-viewport" style={{ height: 520, overflowY: 'auto' }} onScroll={event => setScrollTop(event.currentTarget.scrollTop)}>
    <table aria-label="Order records" className="orders-virtual-table" style={{ tableLayout: 'fixed', width: '100%' }}><thead><tr><th>Order</th><th>Customer</th><th>Channel</th><th>Total</th><th>Status</th><th>Action</th></tr></thead><tbody>
      {window.before > 0 && <tr aria-hidden="true"><td colSpan={6} style={{ height: window.before, padding: 0, border: 0 }} /></tr>}
      {orders.slice(window.start, window.end).map(order => <OrderTableRow key={order.id} order={order} role={role} onSelect={onSelect} onTransition={onTransition} />)}
      {window.after > 0 && <tr aria-hidden="true"><td colSpan={6} style={{ height: window.after, padding: 0, border: 0 }} /></tr>}
    </tbody></table>{!orders.length && <p>No orders match these filters.</p>}</div><div className="toolbar"><span>{orders.length ? window.start + 1 : 0}–{window.end} of {orders.length}</span></div></>;
}
