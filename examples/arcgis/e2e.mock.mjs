import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

const ARCGIS_ASSETS_PREFIX = '/4.30/@arcgis/core/assets/';
const FIXTURE_DIR = 'e2e-fixtures';
const FEATURE_LAYER_URL =
  'https://services.arcgis.com/jIL9msH9OI208GCb/arcgis/rest/services/ACS_Income_2016_5yr/FeatureServer/0';
const STYLE_FIXTURES = new Map([
  ['d7397603e9274052808839b70812be50', 'style-d7397603e9274052808839b70812be50.json'],
  ['4a3922d6d15f405d8c2b7a448a7fbad2', 'style-4a3922d6d15f405d8c2b7a448a7fbad2.json'],
  ['1ddbb25aa29c4811aaadd94de469856a', 'style-1ddbb25aa29c4811aaadd94de469856a.json']
]);
const ITEM_FIXTURES = new Map([
  ['d5dda743788a4b0688fe48f43ae7beb9', 'webmap-item.json'],
  ['9c86eeb5ddac4e6a9f6684539222dfd0', 'feature-item-9c86eeb5ddac4e6a9f6684539222dfd0.json'],
  ['d7397603e9274052808839b70812be50', 'vector-item-d7397603e9274052808839b70812be50.json'],
  ['4a3922d6d15f405d8c2b7a448a7fbad2', 'vector-item-4a3922d6d15f405d8c2b7a448a7fbad2.json'],
  ['1ddbb25aa29c4811aaadd94de469856a', 'vector-item-1ddbb25aa29c4811aaadd94de469856a.json']
]);
const EMPTY_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAFgwJ/l/4P9wAAAABJRU5ErkJggg==',
  'base64'
);

function fulfillJson(route, body) {
  return route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify(body)
  });
}

async function readFixture(examplePath, fileName) {
  const fixture = JSON.parse(
    await readFile(join(examplePath, FIXTURE_DIR, fileName), 'utf8')
  );

  return fixture.data;
}

function useJsonFeatureQueries(featureLayer) {
  return {
    ...featureLayer,
    supportedQueryFormats: 'JSON',
    advancedQueryCapabilities: {
      ...featureLayer.advancedQueryCapabilities,
      supportsCoordinatesQuantization: false
    }
  };
}

async function fulfillArcgisAsset(route, examplePath) {
  const url = new URL(route.request().url());
  const assetPath = decodeURIComponent(url.pathname.slice(ARCGIS_ASSETS_PREFIX.length));

  if (!assetPath || assetPath.includes('..')) {
    await route.abort();
    return;
  }

  const contentType = assetPath.endsWith('.css')
    ? 'text/css'
    : assetPath.endsWith('.json')
      ? 'application/json'
      : assetPath.endsWith('.js')
        ? 'text/javascript'
        : 'application/octet-stream';

  try {
    await route.fulfill({
      status: 200,
      contentType,
      body: await readFile(join(examplePath, 'node_modules', '@arcgis', 'core', 'assets', assetPath))
    });
  } catch {
    await route.abort();
  }
}

async function fulfillPortalRest(route, fixtures) {
  const url = new URL(route.request().url());
  const path = url.pathname.toLowerCase();

  if (path.endsWith('/portals/self')) {
    await fulfillJson(route, fixtures.portalSelf);
    return;
  }

  if (path.endsWith('/data')) {
    await fulfillJson(route, fixtures.webmapData);
    return;
  }

  if (path.includes('/resources/styles/root.json')) {
    const itemId = path.split('/content/items/')[1]?.split('/')[0];
    await fulfillJson(route, fixtures.styles.get(itemId) ?? { version: 8, sources: {}, layers: [] });
    return;
  }

  if (path.includes('/sprites/sprite')) {
    if (path.endsWith('.png')) {
      await route.fulfill({ status: 200, contentType: 'image/png', body: EMPTY_PNG });
      return;
    }

    await fulfillJson(route, {});
    return;
  }

  if (path.includes('/content/items/')) {
    const itemId = path.split('/content/items/')[1]?.split('/')[0];
    await fulfillJson(route, fixtures.items.get(itemId) ?? {});
    return;
  }

  await fulfillJson(route, {});
}

async function fulfillFeatureService(route, fixtures) {
  const url = new URL(route.request().url());
  const path = url.pathname.toLowerCase();

  if (path.endsWith('/query')) {
    if (url.searchParams.get('f') === 'pbf') {
      await route.fulfill({
        status: 200,
        contentType: 'application/x-protobuf',
        body: fixtures.featureQueryPbf
      });
      return;
    }

    await fulfillJson(route, fixtures.featureQuery);
    return;
  }

  if (path.endsWith('/featureserver/0')) {
    await fulfillJson(route, fixtures.featureLayer);
    return;
  }

  await fulfillJson(route, {
    currentVersion: fixtures.featureLayer.currentVersion,
    serviceDescription: fixtures.featureLayer.serviceDescription,
    layers: [{ id: fixtures.featureLayer.id, name: fixtures.featureLayer.name }]
  });
}

async function fulfillEmptyVectorResource(route) {
  const url = new URL(route.request().url());

  if (url.pathname.endsWith('.pbf')) {
    await route.fulfill({ status: 204, contentType: 'application/x-protobuf', body: Buffer.alloc(0) });
    return;
  }

  await fulfillJson(route, {});
}

/**
 * @param {{ page: import('playwright-chromium').Page, examplePath: string }} options
 * @returns {Promise<() => Promise<void>>}
 */
export async function installMockRoutes({ page, examplePath }) {
  const styles = new Map();
  for (const [itemId, fileName] of STYLE_FIXTURES) {
    styles.set(itemId, await readFixture(examplePath, fileName));
  }

  const items = new Map();
  for (const [itemId, fileName] of ITEM_FIXTURES) {
    items.set(itemId, await readFixture(examplePath, fileName));
  }

  const featureLayer = useJsonFeatureQueries(
    await readFixture(examplePath, 'feature-layer-acs-income-0.json')
  );

  const fixtures = {
    portalSelf: await readFixture(examplePath, 'portal-self.json'),
    webmapData: await readFixture(examplePath, 'webmap-data.json'),
    featureLayer,
    featureQuery: await readFixture(examplePath, 'feature-query-acs-income-0.json'),
    featureQueryPbf: await readFile(
      join(examplePath, FIXTURE_DIR, 'feature-query-acs-income-0.pbf')
    ),
    items,
    styles
  };
  const routeHandlers = [];
  const addRoute = async (url, handler) => {
    await page.context().route(url, handler);
    routeHandlers.push([url, handler]);
  };

  await addRoute('https://js.arcgis.com/4.30/@arcgis/core/assets/**', (route) =>
    fulfillArcgisAsset(route, examplePath)
  );
  await addRoute(/https:\/\/(?:www|cdn)\.arcgis\.com\/sharing\/rest\/.*/, (route) =>
    fulfillPortalRest(route, fixtures)
  );
  await addRoute(`${FEATURE_LAYER_URL}/**`, (route) => fulfillFeatureService(route, fixtures));
  await addRoute(FEATURE_LAYER_URL, (route) => fulfillFeatureService(route, fixtures));
  await addRoute(
    'https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer/**',
    fulfillEmptyVectorResource
  );

  return async () => {
    await Promise.all(routeHandlers.map(([url, handler]) => page.context().unroute(url, handler)));
  };
}

export default installMockRoutes;