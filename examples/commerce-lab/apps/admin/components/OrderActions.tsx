import React from 'react';
import type { Order } from './order-types';
export function OrderActions({ order, role, onTransition }: { order: Order; role: string; onTransition: (id: string, status: string) => void }) {
  if (role !== 'admin') return null;
  return <>{order.status === 'paid' && <button onClick={() => onTransition(order.id, 'shipped')}>Ship</button>}{order.status === 'shipped' && <button onClick={() => onTransition(order.id, 'completed')}>Complete</button>}{order.status !== 'refunded' && <button onClick={() => onTransition(order.id, 'refunded')}>Refund</button>}</>;
}
