import { queryProducts, updateProduct } from './index.mjs';
export function parseReceiptUnits(text) {
  if (!/^\d+$/.test(text)) throw new Error('Receipt units must be a positive safe integer.');
  const units = Number(text);
  if (!Number.isSafeInteger(units) || units <= 0) throw new Error('Receipt units must be a positive safe integer.');
  return units;
}
export function receiptPreview(stock, text) {
  if (!Number.isSafeInteger(stock) || stock < 0) throw new Error('Invalid inventory.');
  const received = parseReceiptUnits(text);
  const after = stock + received;
  if (!Number.isSafeInteger(after)) throw new Error('Receipt units exceed the supported inventory.');
  return { before: stock, received, after };
}
export function receiveStock(state, tenant, productId, units) {
  if (!Number.isSafeInteger(units) || units <= 0) throw new Error('Receipt units must be a positive safe integer.');
  const product = queryProducts(state, tenant).find(row => row.id === productId);
  if (!product) throw new Error('Product does not belong to this merchant.');
  return updateProduct(state, tenant, productId, { stock: receiptPreview(product.stock, String(units)).after });
}
export function inventorySummary(state, tenant) {
  if (!state.tenants.some(row => row.id === tenant)) throw new Error('Unknown merchant.');
  return queryProducts(state, tenant).reduce((total, product) => ({
    totalUnits: total.totalUnits + product.stock,
    stockValue: total.stockValue + product.stock * product.price,
    lowStock: total.lowStock + Number(product.stock < 25),
  }), { totalUnits: 0, stockValue: 0, lowStock: 0 });
}
