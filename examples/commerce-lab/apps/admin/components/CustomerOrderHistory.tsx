import React, { useState } from 'react';
import { Badge, money } from '../../../packages/ui';
import { createFixtures } from '../../../packages/domain/index.mjs';
import { Pagination } from '../../../packages/ui/Pagination';
type Order = ReturnType<typeof createFixtures>['orders'][number];
export function CustomerOrderHistory({ orders, onSelect }: { orders: Order[]; onSelect: (id: string) => void }) {
  const [page, setPage] = useState(0);
  const pages = Math.max(1, Math.ceil(orders.length / 10));
  return <>
    <h3>Order history</h3>
    <table aria-label="Customer order history"><thead><tr><th>Order</th><th>Day</th><th>Charged</th><th>Channel</th><th>Status</th></tr></thead>
      <tbody>{orders.slice(page * 10, (page + 1) * 10).map(order => <tr key={order.id}>
        <td><button onClick={() => onSelect(order.id)}>{order.id}</button></td><td>{order.day}</td>
        <td>{money(order.total)}</td><td>{order.channel}</td><td><Badge>{order.status}</Badge></td>
      </tr>)}</tbody>
    </table>
    <Pagination page={page} pages={pages} label="customer orders" onChange={setPage} />
  </>;
}
