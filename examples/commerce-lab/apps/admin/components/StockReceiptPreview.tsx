import React from 'react';
import { receiptPreview } from '../../../packages/domain/inventory.mjs';

export function StockReceiptPreview({ stock, units }: { stock: number; units: string }) {
  let preview;
  try { preview = receiptPreview(stock, units); }
  catch { return <p>Enter a positive whole number to preview this receipt.</p>; }
  return <section aria-label="Stock receipt preview"><h3>Stock after this delivery</h3>
    <dl><dt>Currently available</dt><dd data-testid="receipt-before">{preview.before}</dd>
      <dt>Receiving</dt><dd data-testid="receipt-received">{preview.received}</dd>
      <dt>After confirmation</dt><dd data-testid="receipt-after">{preview.after}</dd></dl>
    <p>Stock changes only when you confirm the receipt.</p>
  </section>;
}
