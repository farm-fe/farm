export function queryOrders(state, tenant, { search = '', status = 'all', channel = 'all', fromDay = 1, toDay = 30 } = {}) {
  if (!state.tenants.some(row => row.id === tenant)) throw new Error('Unknown merchant.');
  if (!['all', 'paid', 'shipped', 'completed', 'refunded'].includes(status) || !['all', 'Store', 'Social', 'Marketplace'].includes(channel) || !Number.isInteger(fromDay) || !Number.isInteger(toDay) || fromDay < 1 || toDay > 30 || fromDay > toDay) throw new Error('Invalid order filters.');
  const normalized = search.trim().toLowerCase();
  const products = new Map(state.products.filter(product => product.tenantId === tenant).map(product => [product.id, product]));
  return state.orders.filter(order => order.tenantId === tenant && (status === 'all' || order.status === status) && (channel === 'all' || order.channel === channel) && order.day >= fromDay && order.day <= toDay && `${order.id} ${order.customer} ${products.get(order.productId)?.sku ?? ''}`.toLowerCase().includes(normalized));
}
export function orderSummary(orders) {
  return orders.reduce((total, order) => ({ orders: total.orders + 1, netRevenue: total.netRevenue + (order.status === 'refunded' ? 0 : order.total), refundedAmount: total.refundedAmount + (order.status === 'refunded' ? order.total : 0), units: total.units + order.quantity }), { orders: 0, netRevenue: 0, refundedAmount: 0, units: 0 });
}
