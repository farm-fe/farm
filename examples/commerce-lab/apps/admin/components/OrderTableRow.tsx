import React from 'react';
import { Badge, money } from '../../../packages/ui';
import { OrderActions } from './OrderActions';
import type { Order } from './order-types';
export function OrderTableRow({ order, role, onSelect, onTransition }: { order: Order; role: string; onSelect: (id: string) => void; onTransition: (id: string, status: string) => void }) {
  return <tr data-testid="order-row" style={{ height: 48 }}><td><button onClick={() => onSelect(order.id)}>{order.id}</button></td><td>{order.customer}</td><td>{order.channel}</td><td>{money(order.total)}</td><td><Badge>{order.status}</Badge></td><td><OrderActions order={order} role={role} onTransition={onTransition} /></td></tr>;
}
