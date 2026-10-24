import React from 'react';

export function ProductFieldsEditor({ name, price, stock, onName, onPrice, onStock }: {
  name: string; price: string; stock: string;
  onName: (value: string) => void;
  onPrice: (value: string) => void;
  onStock: (value: string) => void;
}) {
  return <>
    <label>Product name<input value={name} onChange={event => onName(event.target.value)} /></label>
    <label>Price (USD)<input inputMode="decimal" value={price} onChange={event => onPrice(event.target.value)} /></label>
    <label>Stock quantity<input inputMode="numeric" value={stock} onChange={event => onStock(event.target.value)} /></label>
  </>;
}
