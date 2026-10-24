import React from 'react';
import { money } from '../../../packages/ui';

export type ConnectionValue = { resource: string; count: number; revenue?: number };

export function ConnectionResult({ value }: { value: ConnectionValue }) {
  return <section aria-label="Connection result"><h3>Merchant {value.resource}</h3>
    <dl><dt>{value.resource === 'metrics' ? 'Non-refunded orders' : 'Records returned'}</dt>
      <dd data-testid="connection-record-count">{value.count}</dd>
      {value.revenue !== undefined && <><dt>Net charged</dt><dd data-testid="connection-revenue">{money(value.revenue)}</dd></>}
    </dl><p>Local API simulator response for the current merchant and role.</p></section>;
}
