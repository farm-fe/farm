import React from 'react';
import { CustomerPurchaseSummary } from './CustomerPurchaseSummary';
import type { Customer } from './CustomerList';
export function CustomerProfile({ customer, onClose, children }: {
  customer: Customer; onClose: () => void; children: React.ReactNode;
}) {
  return <section className="panel" aria-label="Customer profile">
    <h2>{customer.name}</h2>
    <CustomerPurchaseSummary customer={customer} />
    <button onClick={onClose}>Close customer profile</button>
    {children}
  </section>;
}
