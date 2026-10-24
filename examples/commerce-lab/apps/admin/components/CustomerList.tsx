import React from 'react';
import { Panel, money } from '../../../packages/ui';
export type Customer = { name: string; orders: number; revenue: number;
  refundedOrders: number; refundedAmount: number; lastDay: number; channels: string[] };
export function CustomerList({ customers, onSelect }: { customers: Customer[]; onSelect: (name: string) => void }) {
  return <Panel title={`${customers.length} customers`}>
    {!customers.length && <p>No customers match this search.</p>}
    <table aria-label="Customers"><thead><tr><th>Customer</th><th>Orders</th><th>Net spend</th><th>Channels</th><th>Latest day</th></tr></thead>
      <tbody>{customers.map(customer => <tr key={customer.name}>
        <td><button aria-label={`View customer ${customer.name}`} onClick={() => onSelect(customer.name)}>{customer.name}</button></td>
        <td>{customer.orders}</td><td>{money(customer.revenue)}</td><td>{customer.channels.join(', ')}</td><td>{customer.lastDay}</td>
      </tr>)}</tbody>
    </table>
  </Panel>;
}
