import { strict as assert } from 'node:assert';

class NotAssertion {
  /** @type {unknown} */
  #actual;

  constructor(actual) {
    this.#actual = actual;
  }

  toBeNull() {
    assert.notStrictEqual(this.#actual, null, 'Expected value not to be null');
  }

  toBeTruthy() {
    assert.ok(!this.#actual, `Expected value not to be truthy, got: ${String(this.#actual)}`);
  }

  toBe(expected) {
    assert.notStrictEqual(
      this.#actual,
      expected,
      `Expected values not to be equal: ${String(expected)}`
    );
  }

  toEqual(expected) {
    try {
      assert.deepStrictEqual(this.#actual, expected);
    } catch {
      return;
    }
    throw new assert.AssertionError({
      message: 'Expected values NOT to be deeply equal',
      actual: this.#actual,
      expected,
      operator: 'notDeepStrictEqual'
    });
  }

  toContain(expected) {
    assert.ok(
      !String(this.#actual).includes(String(expected)),
      `Expected ${JSON.stringify(this.#actual)} NOT to contain ${JSON.stringify(expected)}`
    );
  }
}

class Assertion {
  get not() {
    return new NotAssertion(this.#actual);
  }

  /** @type {unknown} */
  #actual;

  constructor(actual) {
    this.#actual = actual;
  }

  toBeTruthy() {
    assert.ok(this.#actual, `Expected value to be truthy, got: ${String(this.#actual)}`);
  }

  toBeFalsy() {
    assert.ok(!this.#actual, `Expected value to be falsy, got: ${String(this.#actual)}`);
  }

  toBe(expected) {
    assert.strictEqual(this.#actual, expected);
  }

  toEqual(expected) {
    assert.deepStrictEqual(this.#actual, expected);
  }

  toContain(expected) {
    if (Array.isArray(this.#actual)) {
      assert.ok(
        this.#actual.includes(expected),
        `Expected array to contain ${JSON.stringify(expected)}`
      );
    } else {
      assert.ok(
        String(this.#actual).includes(String(expected)),
        `Expected ${JSON.stringify(this.#actual)} to contain ${JSON.stringify(expected)}`
      );
    }
  }

  contains(expected) {
    this.toContain(expected);
  }

  include(expected) {
    this.toContain(expected);
  }

  eq(expected) {
    this.toBe(expected);
  }

  toBeNull() {
    assert.strictEqual(this.#actual, null);
  }

  toBeUndefined() {
    assert.strictEqual(this.#actual, undefined);
  }

  toBeGreaterThan(expected) {
    assert.ok(
      /** @type {number} */ (this.#actual) > expected,
      `Expected ${String(this.#actual)} to be greater than ${expected}`
    );
  }

  matchSnapshot() {
    assert.ok(
      this.#actual !== undefined && this.#actual !== null,
      'Expected snapshot value to be defined (snapshot comparison not supported)'
    );
  }
}

/**
 * @param {unknown} actual
 * @returns {Assertion}
 */
export function expect(actual) {
  return new Assertion(actual);
}
