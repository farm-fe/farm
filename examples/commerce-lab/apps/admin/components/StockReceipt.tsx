import React, { useState } from 'react';
import { parseReceiptUnits } from '../../../packages/domain/inventory.mjs';
import { StockReceiptPreview } from './StockReceiptPreview';
export function StockReceipt({ sku, stock, onReceive, onCancel }: { sku: string; stock: number; onReceive: (units: number) => void; onCancel: () => void }) {
  const [units, setUnits] = useState(''); const [error, setError] = useState('');
  return <form aria-label="Stock receipt" onSubmit={event => {
    event.preventDefault();
    try { onReceive(parseReceiptUnits(units)); setError(''); }
    catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); }
  }}><h2>Receive delivery for {sku}</h2><label>Receipt units<input aria-label="Receipt units" inputMode="numeric" value={units} onChange={event => setUnits(event.target.value)} /></label>
    <StockReceiptPreview stock={stock} units={units} />
    {error && <p role="alert">{error}</p>}<button type="submit">Confirm stock receipt</button><button type="button" onClick={onCancel}>Cancel receipt</button></form>;
}
