/** Convert the displayed USD price into integer cents before updating state. */
export function productFields(name, priceText, stockText) {
  if (!name.trim()) throw new Error('Product name is required');
  if (!/^\d+(?:\.\d{1,2})?$/.test(priceText)) throw new Error('Invalid price');
  const [whole, fraction = ''] = priceText.split('.');
  const price = Number(whole) * 100 + Number(fraction.padEnd(2, '0'));
  if (!Number.isSafeInteger(price)) throw new Error('Invalid price');
  if (!/^\d+$/.test(stockText)) throw new Error('Invalid stock');
  const stock = Number(stockText);
  if (!Number.isSafeInteger(stock)) throw new Error('Invalid stock');
  return { name: name.trim(), price, stock };
}
