import React, { useState } from 'react';
import { useStore } from '../store';
import { queryProducts } from '../../../packages/domain/index.mjs';
import { inventorySummary, receiveStock } from '../../../packages/domain/inventory.mjs';
import { PageTitle } from '../../../packages/ui';
import { StockReceipt } from '../components/StockReceipt';
import { InventorySummary } from '../components/InventorySummary';
import { InventoryFilters } from '../components/InventoryFilters';
import { InventoryTable } from '../components/InventoryTable';
export default function Inventory() {
  const { state, tenant, update, role } = useStore();
  const [lowOnly, setLowOnly] = useState(false);
  const [search, setSearch] = useState(''); const [selected, setSelected] = useState<string | null>(null);
  const products = queryProducts(state, tenant, search).filter(p => !lowOnly || p.stock < 25);
  const summary = inventorySummary(state, tenant);
  const product = queryProducts(state, tenant).find(row => row.id === selected);
  return <><PageTitle eyebrow="Operations / Inventory" title="Keep the shelves ready">Review low stock and receive a new delivery.</PageTitle>
    <InventorySummary summary={summary} />
    <InventoryFilters search={search} lowOnly={lowOnly} onSearch={setSearch} onLowOnly={setLowOnly} />
    {role === 'admin' && product && <StockReceipt key={product.id} sku={product.sku} stock={product.stock} onCancel={() => setSelected(null)} onReceive={units => { update(receiveStock(state, tenant, product.id, units)); setSelected(null); }} />}
    <InventoryTable products={products} canReceive={role === 'admin'} onReceive={setSelected} /></>;
}
