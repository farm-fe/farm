import assert from 'node:assert/strict';

const source = process.argv[1];
const expected = JSON.parse(process.argv[2]);
const { default: actual } = await import(
  `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`
);
assert.deepStrictEqual(actual, expected);

function checkProperties(value, reference) {
  if (reference === null || typeof reference !== 'object') return;
  assert.equal(
    Object.getPrototypeOf(value),
    Array.isArray(reference) ? Array.prototype : Object.prototype
  );
  for (const key of Object.keys(reference)) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    assert.ok(descriptor, `${key} must be an own property`);
    assert.equal(descriptor.enumerable, true);
    assert.equal(descriptor.writable, true);
    assert.equal(descriptor.configurable, true);
    checkProperties(value[key], reference[key]);
  }
}
checkProperties(actual, expected);
assert.equal(Object.prototype.x, undefined);
