/* eslint-disable */
function foo(message) {
  assert.equal(arguments.length, 1);
  assert.equal(typeof arguments[0], 'string');
  bar(message);
}
