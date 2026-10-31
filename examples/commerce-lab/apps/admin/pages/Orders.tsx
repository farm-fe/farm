import React, { useState } from 'react';
import { useStore } from '../store';
import { transitionOrder } from '../../../packages/domain/index.mjs';
import { queryOrders } from '../../../packages/domain/order-query.mjs';
import { PageTitle } from '../../../packages/ui';
import { OrderDetails } from '../components/OrderDetails';
import { OrderFilters } from '../components/OrderFilters';
import { OrderSummary } from '../components/OrderSummary';
import { VirtualOrderList } from '../components/VirtualOrderList';
export default function Orders() {
  const { state, update, tenant, role } = useStore();
  const [filters, setFilters] = useState({ search: '', status: 'all', channel: 'all', fromDay: 1, toDay: 30 });
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState('');
  const orders = queryOrders(state, tenant, filters);
  const selected = state.orders.find(order => order.id === selectedId && order.tenantId === tenant);
  const product = selected && state.products.find(product => product.id === selected.productId && product.tenantId === tenant);
  return <>
    <PageTitle eyebrow="Operations / Orders" title="Every order, accounted for">Review fulfillment and refunds across your sales channels.</PageTitle>
    <OrderFilters value={filters} count={orders.length} onChange={value => { setFilters(value); setSelectedId(null); setError(''); }} />
    <OrderSummary orders={orders} />
    {error && <p role="alert">{error}</p>}
    <VirtualOrderList key={JSON.stringify(filters)} orders={orders} role={role} onSelect={setSelectedId} onTransition={(id, status) => {
      try { update(transitionOrder(state, tenant, id, status)); setError(''); }
      catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); }
    }} />
    {selected && <OrderDetails order={selected} product={product} onClose={() => setSelectedId(null)} />}
  </>;
}
