import React, { useState } from 'react';
import { useStore } from '../store';
import { PageTitle } from '../../../packages/ui';
import { customerProfiles, customerOrders } from '../../../packages/domain/customers.mjs';
import { CustomerList } from '../components/CustomerList';
import { CustomerProfile } from '../components/CustomerProfile';
import { CustomerOrderHistory } from '../components/CustomerOrderHistory';
import { OrderDetails } from '../components/OrderDetails';
export default function Customers() {
  const { state, tenant } = useStore(); const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [orderId, setOrderId] = useState<string | null>(null);
  const rows = customerProfiles(state, tenant, search);
  const customer = customerProfiles(state, tenant).find(row => row.name === selected);
  const order = state.orders.find(row => row.tenantId === tenant && row.id === orderId && row.customer === selected);
  return <><PageTitle eyebrow="Audience / Customers" title="Know the people behind the orders">Revenue and channel attribution from merchant-owned order history.</PageTitle>
    <input aria-label="Search customers" value={search} onChange={e => setSearch(e.target.value)} placeholder="Search customer" />
    <CustomerList customers={rows} onSelect={name => { setSelected(name); setOrderId(null); }} />
    {customer && <CustomerProfile customer={customer} onClose={() => { setSelected(null); setOrderId(null); }}>
      <CustomerOrderHistory key={customer.name} orders={customerOrders(state, tenant, customer.name)} onSelect={setOrderId} />
    </CustomerProfile>}
    {order && <OrderDetails order={order} product={state.products.find(product => product.id === order.productId && product.tenantId === tenant)} onClose={() => setOrderId(null)} />}
  </>;
}
