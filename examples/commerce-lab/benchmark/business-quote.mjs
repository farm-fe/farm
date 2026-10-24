/** Prices and shipping are integer cents; discounts share one category cap. */
export function businessQuote({ price, stock, quantity, discount, destination, rules, tiers, shipping }) {
  if (!Number.isSafeInteger(price) || price <= 0) throw new RangeError('Invalid price');
  if (!Number.isSafeInteger(quantity) || quantity < 1 || !Number.isSafeInteger(price * quantity)) throw new RangeError('Invalid quantity');
  if (!Number.isSafeInteger(stock) || quantity > stock - rules.reserveStock) throw new RangeError('Insufficient stock after reservation');
  if (!Number.isFinite(discount) || discount < 0 || discount > rules.maxDiscount) throw new RangeError('Category discount limit exceeded');
  const route = shipping.find(row => row.destination === destination);
  if (!route) throw new RangeError('Unknown destination');
  const tier = tiers.filter(row => quantity >= Number(row.minQuantity)).sort((a, b) => Number(b.minQuantity) - Number(a.minQuantity))[0];
  const effectiveDiscount = Math.min(rules.maxDiscount, discount + Number(tier?.discount ?? 0));
  const subtotal = price * quantity;
  const merchandise = Math.round(subtotal * (100 - effectiveDiscount) / 100);
  const freight = merchandise >= Number(route.freeAbove) ? 0 : Number(route.cents);
  if (!Number.isSafeInteger(freight) || freight < 0 || !Number.isSafeInteger(merchandise + freight)) throw new RangeError('Invalid shipping quote');
  return { subtotal, discount: effectiveDiscount, shipping: freight, total: merchandise + freight };
}
