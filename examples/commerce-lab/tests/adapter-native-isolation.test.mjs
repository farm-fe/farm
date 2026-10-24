import assert from 'node:assert/strict';
import test from 'node:test';
import { channelVirtual } from '../adapters/channel-virtual.mjs';
import { productUrl } from '../adapters/product-url.mjs';
import { productImageDom } from '../adapters/product-image.mjs';
import { commerceIcons } from '../adapters/commerce-icons.mjs';
import { publicationStrip } from '../adapters/publication-strip.mjs';
import { modularUi } from '../adapters/modular-ui.mjs';
import { reactComponents } from '../adapters/react-components.mjs';

const adapters = [
  ['channelVirtual', channelVirtual, options => assert.match(options['commerce-channel-capabilities.js'], /Marketplace/)],
  ['productUrl', productUrl, options => { assert.equal(options.limit, 550); assert.equal(options.emitFiles, true); }],
  ['productImageDom', productImageDom, options => assert.equal(options.dom, true)],
  ['commerceIcons', commerceIcons, options => { assert.equal(options.autoInstall, false); assert.equal(options.compiler, 'jsx'); }],
  ['publicationStrip', (group, factory) => publicationStrip(group, 'production', factory), options => assert.deepEqual(options.functions, ['commerceDiagnostics\\.trace'])],
  ['modularUi', modularUi, options => assert.equal(options.libraryName, '@arco-design/web-react')],
  ['reactComponents', reactComponents, options => { assert.equal(options.local, true); assert.equal(options.dts, false); }],
];

for (const [name, adapter, checkOptions] of adapters) {
  test(`${name}: Rust and mixed paths require an explicitly supplied native factory`, () => {
    for (const group of ['rust', 'mixed']) {
      assert.throws(() => adapter(group), error => error instanceof TypeError && error.message.includes(name) && error.message.includes('native plugin factory'));
      let calls = 0;
      const factoryResult = Symbol('factory delegation result');
      assert.equal(adapter(group, options => {
        calls++;
        checkOptions(options);
        return factoryResult;
      }), factoryResult);
      assert.equal(calls, 1);
    }
  });
}

test('development strip remains disabled without loading or invoking native code', () => {
  for (const group of ['js', 'rust', 'mixed']) {
    assert.equal(publicationStrip(group, 'development', () => { throw Error('native factory must not run'); }), false);
  }
});

test('JS adapter branches never invoke a supplied native factory', () => {
  for (const [name, adapter] of adapters) {
    const plugin = adapter('js', () => { throw Error(`${name} loaded native code`); });
    assert.equal(typeof plugin.name, 'string');
    assert.ok(plugin.name.startsWith('commerce-js-'));
  }
});