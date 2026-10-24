import React from 'react';
import { ReportingFilters } from './ReportingFilters';
import type { OrderFiltersValue } from './order-types';
export function OrderFilters({ value, count, onChange }: { value: OrderFiltersValue; count: number; onChange: (value: OrderFiltersValue) => void }) {
  return <><div className="toolbar"><input aria-label="Search orders" placeholder="Order ID, customer or product SKU" value={value.search} onChange={event => onChange({ ...value, search: event.target.value })} />
    <select aria-label="Order status" value={value.status} onChange={event => onChange({ ...value, status: event.target.value })}><option value="all">All statuses</option>{['paid', 'shipped', 'completed', 'refunded'].map(status => <option key={status}>{status}</option>)}</select><span>{count} orders</span></div>
    <ReportingFilters fromDay={value.fromDay} toDay={value.toDay} channel={value.channel} onChange={range => onChange({ ...value, ...range })} /></>;
}
