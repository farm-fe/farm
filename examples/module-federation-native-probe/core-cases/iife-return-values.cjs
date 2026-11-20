const fulfilled = Promise.resolve(42);
const pending = new Promise(() => {});
let thenReads = 0;
const thenable = { get then() { thenReads++; return resolve => resolve(42); } };
const returnedFulfilled = (function () { require('./async.js'); return fulfilled; })();
const returnedPending = (function () { require('./async.js'); return pending; })();
const returnedThenable = (function () { require('./async.js'); return thenable; })();
const fallthrough = (function () { require('./async.js'); })();
const bare = (function () { require('./async.js'); return; })();
const fromFinally = (function () {
  require('./async.js');
  try { return 7; } finally { return fulfilled; }
})();
const nested = (function () {
  require('./async.js');
  function inner() { return fulfilled; }
  return inner();
})();
const computed = (function () {
  return { get [require('./async.js').value]() { return 7; } };
})();
const syncAccessor = { get value() { return require('./sync.cjs').value; } };
// The Proxy records the receiver independently of directive stripping during minification.
const directCallThisUndefined = (function () {
  require('./async.js');
  return new Proxy(function () {}, { apply(_target, thisArg) { return thisArg === undefined; } });
})()();
const taggedThisUndefined = (function () {
  require('./async.js');
  return new Proxy(function () {}, { apply(_target, thisArg) { return thisArg === undefined; } });
})()`tag`;
module.exports = {
  directCallThisUndefined,
  taggedThisUndefined,
  fulfilledIdentity: returnedFulfilled === fulfilled,
  pendingIdentity: returnedPending === pending,
  thenableIdentity: returnedThenable === thenable,
  thenReads,
  fallthroughUndefined: fallthrough === undefined,
  bareUndefined: bare === undefined,
  finallyIdentity: fromFinally === fulfilled,
  nestedIdentity: nested === fulfilled,
  computedKey: computed[42],
  syncAccessor: syncAccessor.value,
};
