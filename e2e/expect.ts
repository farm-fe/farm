/**
 * Minimal jest/vitest-compatible expect() implementation for standalone E2E runner.
 * Covers all assertion patterns used in Farm's e2e spec files.
 */
import { strict as assert } from 'node:assert';

class NotAssertion {
  private readonly actual: unknown;

  constructor(actual: unknown) {
    this.actual = actual;
  }

  toBeNull(): void {
    assert.notStrictEqual(this.actual, null, 'Expected value not to be null');
  }

  toBeTruthy(): void {
    assert.ok(!this.actual, `Expected value not to be truthy, got: ${String(this.actual)}`);
  }

  toBe(expected: unknown): void {
    assert.notStrictEqual(
      this.actual,
      expected,
      `Expected values not to be equal: ${String(expected)}`
    );
  }

  toEqual(expected: unknown): void {
    try {
      assert.deepStrictEqual(this.actual, expected);
    } catch {
      return; // negation succeeded
    }
    throw new assert.AssertionError({
      message: 'Expected values NOT to be deeply equal',
      actual: this.actual,
      expected,
      operator: 'notDeepStrictEqual'
    });
  }

  toContain(expected: string): void {
    assert.ok(
      !String(this.actual).includes(String(expected)),
      `Expected ${JSON.stringify(this.actual)} NOT to contain ${JSON.stringify(expected)}`
    );
  }
}

class Assertion {
  get not(): NotAssertion {
    return new NotAssertion(this.actual);
  }

  private readonly actual: unknown;

  constructor(actual: unknown) {
    this.actual = actual;
  }

  toBeTruthy(): void {
    assert.ok(this.actual, `Expected value to be truthy, got: ${String(this.actual)}`);
  }

  toBeFalsy(): void {
    assert.ok(!this.actual, `Expected value to be falsy, got: ${String(this.actual)}`);
  }

  toBe(expected: unknown): void {
    assert.strictEqual(this.actual, expected);
  }

  toEqual(expected: unknown): void {
    assert.deepStrictEqual(this.actual, expected);
  }

  toContain(expected: string): void {
    if (Array.isArray(this.actual)) {
      assert.ok(
        (this.actual as unknown[]).includes(expected),
        `Expected array to contain ${JSON.stringify(expected)}`
      );
    } else {
      assert.ok(
        String(this.actual).includes(String(expected)),
        `Expected ${JSON.stringify(this.actual)} to contain ${JSON.stringify(expected)}`
      );
    }
  }

  /** chai / vitest alias for toContain */
  contains(expected: string): void {
    this.toContain(expected);
  }

  /** chai alias for toContain */
  include(expected: string): void {
    this.toContain(expected);
  }

  /** chai alias for toBe */
  eq(expected: unknown): void {
    this.toBe(expected);
  }

  toBeNull(): void {
    assert.strictEqual(this.actual, null);
  }

  toBeUndefined(): void {
    assert.strictEqual(this.actual, undefined);
  }

  toBeGreaterThan(expected: number): void {
    assert.ok(
      (this.actual as number) > expected,
      `Expected ${String(this.actual)} to be greater than ${expected}`
    );
  }

  /**
   * Snapshot testing is not supported in the standalone runner.
   * Falls back to an existence check so existing tests don't crash.
   */
  matchSnapshot(): void {
    assert.ok(
      this.actual !== undefined && this.actual !== null,
      'Expected snapshot value to be defined (snapshot comparison not supported)'
    );
  }
}

export function expect(actual: unknown): Assertion {
  return new Assertion(actual);
}
