/** @typedef {{ id: string, tenantId: string, name: string, discount: number, productIds: string[], status: string }} Campaign */
import { campaignPolicy } from './campaign-policy.mjs';
/** Deterministic business data. Prices are integer cents. */
export function createFixtures(seed = 42, { productsPerTenant = 60, ordersPerTenant = 400 } = {}) {
  for (const count of [productsPerTenant, ordersPerTenant]) {
    if (!Number.isInteger(count) || count < 1 || count > 100000) {
      throw new RangeError('Fixture sizes must be integers from 1 to 100000');
    }
  }
  let value = seed >>> 0;
  const random = () => {
    value = (Math.imul(value, 1664525) + 1013904223) >>> 0;
    return value / 4294967296;
  };
  const tenants = [
    { id: 'north', name: 'North Atelier' },
    { id: 'south', name: 'South Supply' },
  ];
  const categories = ['Home', 'Outdoor', 'Studio', 'Travel'];
  const products = tenants.flatMap((tenant) =>
    Array.from({ length: productsPerTenant }, (_, i) => ({
      id: `${tenant.id}-p${i}`,
      tenantId: tenant.id,
      sku: `${tenant.id.toUpperCase()}-${1000 + i}`,
      name: `${categories[i % 4]} ${['Canvas', 'Ceramic', 'Linen', 'Essential'][i % 4]} ${i + 1}`,
      category: categories[i % 4],
      price: 1000 + Math.floor(random() * 15000),
      stock: Math.floor(random() * 200),
      status: i % 5 === 0 ? 'draft' : 'published',
    })),
  );
  const productsById = new Map(products.map(product => [product.id, product]));
  const orders = tenants.flatMap((tenant) =>
    Array.from({ length: ordersPerTenant }, (_, i) => {
      const product = productsById.get(`${tenant.id}-p${i % productsPerTenant}`);
      const quantity = 1 + Math.floor(random() * 4);
      return {
        id: `${tenant.id}-o${i}`,
        tenantId: tenant.id,
        productId: product.id,
        customer: `Customer ${1 + (i % 97)}`,
        quantity,
        total: product.price * quantity,
        day: 1 + (i % 30),
        channel: ['Store', 'Social', 'Marketplace'][i % 3],
        status: ['paid', 'shipped', 'completed', 'refunded'][i % 4],
      };
    }),
  );
  return {
    schema: 1,
    tenants,
    products,
    orders,
    campaigns: /** @type {Campaign[]} */ ([]),
  };
}

function tenantExists(state, tenantId) {
  if (!state.tenants.some((t) => t.id === tenantId))
    throw new Error('Tenant not found');
}

/**
 * @param {ReturnType<typeof createFixtures>} state
 * @param {string} tenantId
 * @param {string} search
 */
export function queryProducts(state, tenantId, search = '') {
  const normalized = search.toLowerCase();
  return state.products.filter(
    (p) =>
      p.tenantId === tenantId &&
      `${p.sku} ${p.name} ${p.category}`.toLowerCase().includes(normalized),
  );
}

export function validateImport(rows, existing = []) {
  const seen = new Set(existing.map((p) => p.sku));
  const valid = [],
    errors = [];
  rows.forEach((row, index) => {
    const sku = String(row.sku ?? '').trim();
    const name = String(row.name ?? '').trim();
    const priceText = String(row.price ?? '').trim();
    const price = Number(priceText);
    const add = (field, message) =>
      errors.push({ row: index + 2, field, message });
    const start = errors.length;
    if (!sku || seen.has(sku)) add('sku', 'SKU is missing or already exists');
    if (!name) add('name', 'Product name is required');
    if (
      !/^\d+(\.\d{1,2})?$/.test(priceText) ||
      !Number.isFinite(price) ||
      price > 1e7
    )
      add('price', 'Price must be 0–10000000 with up to two decimals');
    if (sku) seen.add(sku);
    if (errors.length === start)
      valid.push({
        sku,
        name,
        price: Math.round(price * 100),
        category: String(row.category || 'Home'),
        stock: 0,
        status: 'draft',
      });
  });
  return { valid, errors };
}

export function importProducts(state, tenantId, rows) {
  tenantExists(state, tenantId);
  const result = validateImport(rows, queryProducts(state, tenantId));
  if (result.errors.length) throw new Error('Import validation failed');
  const products = result.valid.map((p, index) => ({
    ...p,
    tenantId,
    id: `${tenantId}-import-${state.products.length + index}`,
  }));
  return { ...state, products: [...state.products, ...products] };
}

export function updateProduct(state, tenantId, id, changes) {
  tenantExists(state, tenantId);
  const product = state.products.find(
    (p) => p.tenantId === tenantId && p.id === id,
  );
  if (!product) throw new Error('Product not found');
  const allowed = {
    name: changes.name ?? product.name,
    price: changes.price ?? product.price,
    status: changes.status ?? product.status,
    stock: changes.stock ?? product.stock,
  };
  if (
    !allowed.name.trim() ||
    !Number.isSafeInteger(allowed.price) ||
    allowed.price < 0 ||
    !Number.isSafeInteger(allowed.stock) ||
    allowed.stock < 0 ||
    !['draft', 'published'].includes(allowed.status)
  )
    throw new Error('Invalid product update');
  return {
    ...state,
    products: state.products.map((p) =>
      p.id === id ? { ...p, ...allowed } : p,
    ),
  };
}

/** Publish a validated selection in one state update; no partial mutations. */
export function publishProducts(state, tenantId, ids) {
  tenantExists(state, tenantId);
  if (!Array.isArray(ids) || !ids.length) throw new Error('Select products to publish');
  const selected = new Set(ids);
  if (selected.size !== ids.length) throw new Error('Duplicate product selection');
  const owned = new Set(state.products.filter(product => product.tenantId === tenantId).map(product => product.id));
  if (ids.some(id => !owned.has(id))) throw new Error('Product not found');
  return {
    ...state,
    products: state.products.map(product => selected.has(product.id) ? { ...product, status: 'published' } : product),
  };
}

export function transitionOrder(state, tenantId, id, status) {
  tenantExists(state, tenantId);
  const order = state.orders.find(
    (o) => o.id === id && o.tenantId === tenantId,
  );
  if (!order) throw new Error('Order not found');
  const transitions = {
    paid: ['shipped', 'refunded'],
    shipped: ['completed', 'refunded'],
    completed: ['refunded'],
    refunded: [],
  };
  if (!transitions[order.status]?.includes(status))
    throw new Error('Invalid order transition');
  return {
    ...state,
    orders: state.orders.map((o) => (o.id === id ? { ...o, status } : o)),
  };
}

export function createCampaign(state, tenantId, input) {
  tenantExists(state, tenantId);
  if (
    !Number.isInteger(input.discount) ||
    input.discount <= 0 ||
    input.discount > campaignPolicy(state, tenantId).maxDiscount
  )
    throw new Error('Invalid discount');
  if (!input.name?.trim()) throw new Error('Campaign name is required');
  if (
    !Array.isArray(input.productIds) ||
    !input.productIds.length ||
    new Set(input.productIds).size !== input.productIds.length ||
    input.productIds.some(
      (id) =>
        !state.products.some(
          (p) =>
            p.id === id && p.tenantId === tenantId && p.status === 'published',
        ),
    )
  )
    throw new Error('Invalid campaign product');
  const campaign = {
    ...input,
    name: input.name.trim(),
    productIds: [...input.productIds],
    id: `${tenantId}-c${state.campaigns.length}`,
    tenantId,
    status: 'published',
  };
  return { ...state, campaigns: [...state.campaigns, campaign] };
}

export function metrics(state, tenantId, { fromDay = 1, toDay = 30, channel = 'all' } = {}) {
  if (!Number.isInteger(fromDay) || !Number.isInteger(toDay) || fromDay < 1 || toDay > 30 || fromDay > toDay) {
    throw new Error('Invalid reporting range');
  }
  if (!['all', 'Store', 'Social', 'Marketplace'].includes(channel)) throw new Error('Invalid reporting channel');
  const orders = state.orders.filter(
    (o) => o.tenantId === tenantId && o.status !== 'refunded' && o.day >= fromDay && o.day <= toDay && (channel === 'all' || o.channel === channel),
  );
  const days = Array(30).fill(0);
  const channels = ['Store', 'Social', 'Marketplace'].map(channel => ({ channel, revenue: 0, orders: 0 }));
  for (const order of orders) {
    days[order.day - 1] += order.total;
    const summary = channels.find(row => row.channel === order.channel);
    if (summary) { summary.revenue += order.total; summary.orders += 1; }
  }
  return {
    revenue: orders.reduce((sum, o) => sum + o.total, 0),
    orders: orders.length,
    products: state.products.filter(
      (p) => p.tenantId === tenantId && p.status === 'published',
    ).length,
    campaigns: state.campaigns.filter((c) => c.tenantId === tenantId).length,
    days,
    channels,
  };
}
