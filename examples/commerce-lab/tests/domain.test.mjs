import assert from 'node:assert/strict';
import test from 'node:test';
import {
  createFixtures,
  validateImport,
  importProducts,
  transitionOrder,
  createCampaign,
  metrics,
  queryProducts,
  updateProduct,
  publishProducts,
} from '../packages/domain/index.mjs';

test('bulk publication is atomic and respects merchant ownership', () => {
  const state = createFixtures();
  const next = publishProducts(state, 'north', ['north-p0', 'north-p5']);
  assert.equal(next.products.find(p => p.id === 'north-p0').status, 'published');
  assert.equal(next.products.find(p => p.id === 'north-p5').status, 'published');
  assert.equal(next.products.find(p => p.id === 'south-p0').status, 'draft');
  assert.equal(state.products.find(p => p.id === 'north-p0').status, 'draft');
  assert.throws(() => publishProducts(state, 'north', ['north-p0', 'south-p0']), /not found/i);
  assert.throws(() => publishProducts(state, 'north', []), /select/i);
  assert.throws(() => publishProducts(state, 'north', ['north-p0', 'north-p0']), /duplicate/i);
});

test('overview range and channel filters share one revenue and chart scope', () => {
  const state = { products: [], campaigns: [], orders: [
    { tenantId: 'north', channel: 'Store', status: 'paid', day: 1, total: 1250 },
    { tenantId: 'north', channel: 'Social', status: 'paid', day: 3, total: 2500 },
    { tenantId: 'north', channel: 'Store', status: 'completed', day: 3, total: 1000 },
    { tenantId: 'north', channel: 'Store', status: 'refunded', day: 3, total: 9999 },
    { tenantId: 'south', channel: 'Store', status: 'paid', day: 3, total: 8000 },
  ] };
  const totals = metrics(state, 'north', { fromDay: 2, toDay: 4, channel: 'Store' });
  assert.equal(totals.orders, 1);
  assert.equal(totals.revenue, 1000);
  assert.equal(totals.days.reduce((sum, value) => sum + value, 0), 1000);
  assert.equal(totals.days[2], 1000);
  assert.deepEqual(totals.channels, [
    { channel: 'Store', revenue: 1000, orders: 1 },
    { channel: 'Social', revenue: 0, orders: 0 },
    { channel: 'Marketplace', revenue: 0, orders: 0 },
  ]);
  const all = metrics(state, 'north', { fromDay: 2, toDay: 4 });
  assert.equal(all.channels.reduce((sum, row) => sum + row.revenue, 0), all.revenue);
  assert.equal(all.channels.reduce((sum, row) => sum + row.orders, 0), all.orders);
  assert.throws(() => metrics(state, 'north', { fromDay: 4, toDay: 2 }), /range/i);
  assert.throws(() => metrics(state, 'north', { fromDay: 1.5 }), /range/i);
  assert.throws(() => metrics(state, 'north', { channel: 'Unknown' }), /channel/i);
});

test('scaled fixtures preserve deterministic product relationships for both tenants', () => {
  const state = createFixtures(42, { productsPerTenant: 120, ordersPerTenant: 10000 });
  assert.deepEqual(state, createFixtures(42, { productsPerTenant: 120, ordersPerTenant: 10000 }));
  assert.equal(state.products.length, 240);
  assert.equal(state.orders.length, 20000);
  const products = new Map(state.products.map(product => [product.id, product]));
  for (const order of state.orders) {
    const product = products.get(order.productId);
    assert.equal(product.tenantId, order.tenantId);
    assert.equal(order.total, product.price * order.quantity);
  }
  assert.equal(new Set(state.orders.map(order => order.id)).size, 20000);
});
test('fixture sizes reject invalid and unbounded allocations', () => {
  for (const size of [0, -1, 1.5, NaN, Infinity, 100001]) {
    assert.throws(() => createFixtures(42, { productsPerTenant: size }), RangeError);
    assert.throws(() => createFixtures(42, { ordersPerTenant: size }), RangeError);
  }
});

test('fixtures are deterministic and tenant-owned', () => {
  const state = createFixtures(42);
  assert.deepEqual(state, createFixtures(42));
  assert.equal(state.products.length, 120);
  assert.equal(state.orders.length, 800);
  assert.equal(new Set(state.tenants.map((t) => t.id)).size, 2);
  assert.equal(queryProducts(state, 'north').length, 60);
  assert.ok(
    state.orders.every((order) =>
      state.products.some(
        (p) => p.id === order.productId && p.tenantId === order.tenantId,
      ),
    ),
  );
});

test('import validation reports row-specific errors and treats zero as a valid price', () => {
  const state = createFixtures();
  const rows = [
    { sku: 'NEW', name: 'New product', price: '0' },
    { sku: 'NEW', name: '', price: '-1' },
    { sku: state.products[0].sku, name: 'Duplicate', price: '12' },
  ];
  const result = validateImport(rows, queryProducts(state, 'north'));
  assert.equal(result.valid.length, 1);
  assert.deepEqual(
    result.errors.map((e) => [e.row, e.field]),
    [
      [3, 'sku'],
      [3, 'name'],
      [3, 'price'],
      [4, 'sku'],
    ],
  );
});

test('invalid imports are atomic and valid imports do not affect another tenant', () => {
  const state = createFixtures();
  assert.throws(
    () =>
      importProducts(state, 'north', [{ sku: 'N', name: 'New', price: 'bad' }]),
    /validation/,
  );
  const next = importProducts(state, 'north', [
    { sku: 'N', name: 'New', price: '12.50' },
  ]);
  assert.equal(next.products.length, state.products.length + 1);
  assert.equal(next.products.at(-1).price, 1250);
  assert.deepEqual(queryProducts(next, 'south'), queryProducts(state, 'south'));
  assert.equal(state.products.length, 120);
});

test('order transitions protect tenant boundaries and reject invalid transitions', () => {
  const state = createFixtures();
  const order = state.orders.find(
    (o) => o.tenantId === 'north' && o.status === 'paid',
  );
  assert.throws(
    () => transitionOrder(state, 'south', order.id, 'shipped'),
    /not found/,
  );
  assert.throws(
    () => transitionOrder(state, 'north', order.id, 'paid'),
    /transition/,
  );
  const next = transitionOrder(state, 'north', order.id, 'shipped');
  assert.equal(next.orders.find((o) => o.id === order.id).status, 'shipped');
  assert.equal(order.status, 'paid');
});

test('campaigns validate product ownership and discount rules', () => {
  const state = createFixtures();
  assert.throws(
    () =>
      createCampaign(state, 'north', {
        name: 'Sale',
        discount: 120,
        productIds: [],
      }),
    /discount/,
  );
  const foreign = state.products.find((p) => p.tenantId === 'south');
  assert.throws(
    () =>
      createCampaign(state, 'north', {
        name: 'Sale',
        discount: 10,
        productIds: [foreign.id],
      }),
    /product/,
  );
  const published = state.products.find(
    (p) => p.tenantId === 'north' && p.status === 'published',
  );
  const next = createCampaign(state, 'north', {
    name: 'Summer',
    discount: 10,
    productIds: [published.id],
  });
  assert.equal(next.campaigns.at(-1).status, 'published');
});

test('campaign selection rejects duplicate products and fractional discounts without mutation', () => {
  const state = createFixtures();
  const product = state.products.find(p => p.tenantId === 'north' && p.status === 'published');
  const before = JSON.stringify(state);
  assert.throws(() => createCampaign(state, 'north', { name: 'Sale', discount: 10.5, productIds: [product.id] }), /discount/);
  assert.throws(() => createCampaign(state, 'north', { name: 'Sale', discount: 10, productIds: [product.id, product.id] }), /product/);
  assert.equal(JSON.stringify(state), before);
  const ids = [product.id];
  const next = createCampaign(state, 'north', { name: '  Selected launch  ', discount: 10, productIds: ids });
  ids.push('foreign');
  assert.equal(next.campaigns.at(-1).name, 'Selected launch');
  assert.deepEqual(next.campaigns.at(-1).productIds, [product.id]);
});

test('metrics exclude refunded orders and use integer cents', () => {
  const state = {
    orders: [
      { tenantId: 'north', status: 'paid', total: 1250, day: 1 },
      { tenantId: 'north', status: 'refunded', total: 9999, day: 1 },
      { tenantId: 'south', status: 'paid', total: 5000, day: 2 },
    ],
    products: [{ tenantId: 'north', status: 'published' }],
    campaigns: [],
  };
  assert.equal(metrics(state, 'north').revenue, 1250);
  assert.equal(metrics(state, 'north').days[0], 1250);
});

test('product edits reject foreign merchants and invalid money or stock', () => {
  const state = createFixtures();
  const product = state.products[0];
  assert.throws(() => updateProduct(state, 'south', product.id, { name: 'Wrong tenant' }), /not found/);
  assert.throws(() => updateProduct(state, 'north', product.id, { price: 12.5 }), /Invalid/);
  assert.throws(() => updateProduct(state, 'north', product.id, { stock: -1 }), /Invalid/);
  const next = updateProduct(state, 'north', product.id, { stock: 20, status: 'published' });
  assert.equal(next.products[0].stock, 20);
  assert.equal(next.products[0].status, 'published');
  assert.equal(state.products[0].status, 'draft');
});
