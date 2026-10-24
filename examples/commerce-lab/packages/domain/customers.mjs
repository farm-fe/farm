function requireMerchant(state, tenantId) {
  if (!state.tenants.some(tenant => tenant.id === tenantId)) throw new Error('Unknown merchant');
}
export function customerProfiles(state, tenantId, search = '') {
  requireMerchant(state, tenantId);
  const profiles = new Map();
  for (const order of state.orders) {
    if (order.tenantId !== tenantId) continue;
    const profile = profiles.get(order.customer) ?? { name: order.customer, orders: 0,
      revenue: 0, refundedOrders: 0, refundedAmount: 0, lastDay: 0, channels: [] };
    profile.orders++;
    profile.lastDay = Math.max(profile.lastDay, order.day);
    if (!profile.channels.includes(order.channel)) profile.channels.push(order.channel);
    if (order.status === 'refunded') { profile.refundedOrders++; profile.refundedAmount += order.total; }
    else profile.revenue += order.total;
    profiles.set(order.customer, profile);
  }
  const normalized = search.trim().toLowerCase();
  return [...profiles.values()].filter(profile => profile.name.toLowerCase().includes(normalized))
    .map(profile => ({ ...profile, channels: profile.channels.sort() }))
    .sort((a, b) => b.revenue - a.revenue || a.name.localeCompare(b.name, 'en'));
}
export function customerOrders(state, tenantId, name) {
  requireMerchant(state, tenantId);
  return state.orders.filter(order => order.tenantId === tenantId && order.customer === name)
    .sort((a, b) => b.day - a.day || a.id.localeCompare(b.id, 'en'));
}
