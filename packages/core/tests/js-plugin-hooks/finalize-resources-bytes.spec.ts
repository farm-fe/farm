import { expect, test } from 'vitest';
import type { Resource } from '../../src/index.js';
import { getCompiler } from './common.js';

test('finalizeResources rejects parameter encoding errors without leaving the compiler waiting', async () => {
  const previousBytes = Object.getOwnPropertyDescriptor(
    Object.prototype,
    'bytes'
  );
  let reachedConsumer = false;
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'throwing-byte-setter',
        priority: 1000,
        finalizeResources: {
          executor: ({ resourcesMap }) => {
            Object.defineProperty(Object.prototype, 'bytes', {
              configurable: true,
              set() {
                throw new Error('fixture-byte-setter-failure');
              }
            });
            return resourcesMap;
          }
        }
      },
      {
        name: 'unreachable-consumer',
        priority: 900,
        finalizeResources: {
          executor: () => {
            reachedConsumer = true;
            return null;
          }
        }
      }
    ],
    'augment-resource-hash-generate-resources'
  );
  try {
    await expect(compiler.compile()).rejects.toThrow(
      'Cannot encode hook parameters'
    );
    expect(reachedConsumer).toBe(false);
  } finally {
    if (previousBytes)
      Object.defineProperty(Object.prototype, 'bytes', previousBytes);
    else Reflect.deleteProperty(Object.prototype, 'bytes');
  }
}, 10000);

test.each([false, true])(
  'finalizeResources preserves binary arrays and metadata (async=%s)',
  async (asyncResult) => {
    const bytes = Array.from({ length: 256 * 4096 }, (_, index) => index % 256);
    const specialKey = '路径/different-from-resource-name';
    let expected: Resource & { specialPlaceholders: Record<string, string> };
    let observed = false;
    const previousBytes = Object.getOwnPropertyDescriptor(
      Object.prototype,
      'bytes'
    );
    const inheritedWrites: number[] = [];
    const compiler = await getCompiler(
      '',
      [
        {
          name: 'binary-resource-producer',
          priority: 1000,
          finalizeResources: {
            executor: ({ resourcesMap }) => {
              const original = Object.values(resourcesMap)[0];
              expected = {
                ...original,
                bytes,
                name: 'binary-resource',
                specialPlaceholders: { format: 'esm', unicode: '业务' }
              };
              Object.defineProperty(Object.prototype, 'bytes', {
                configurable: true,
                set(value: number[]) {
                  inheritedWrites.push(value.length);
                  Object.defineProperty(this, 'bytes', {
                    value,
                    configurable: true,
                    enumerable: true,
                    writable: true
                  });
                }
              });
              return {
                [specialKey]: expected,
                empty: { ...expected, bytes: [] }
              };
            }
          }
        },
        {
          name: 'binary-resource-consumer',
          priority: 900,
          finalizeResources: {
            executor: ({ resourcesMap, config }) => {
              expect(inheritedWrites.sort((a, b) => a - b)).toEqual([
                0,
                bytes.length
              ]);
              const actual = resourcesMap[specialKey];
              expect(config.input).toBeTruthy();
              expect(Array.isArray(actual.bytes)).toBe(true);
              expect(Object.getPrototypeOf(actual.bytes)).toBe(Array.prototype);
              expect(Buffer.from(actual.bytes)).toEqual(Buffer.from(bytes));
              expect({ ...actual, bytes: [] }).toEqual({
                ...expected,
                bytes: []
              });
              expect(resourcesMap.empty.bytes).toEqual([]);
              expect(Object.hasOwn(actual, 'meta')).toBe(false);
              observed = true;
              return asyncResult ? Promise.resolve(resourcesMap) : resourcesMap;
            }
          }
        }
      ],
      'augment-resource-hash-generate-resources'
    );
    try {
      await compiler.compile();
    } finally {
      if (previousBytes)
        Object.defineProperty(Object.prototype, 'bytes', previousBytes);
      else Reflect.deleteProperty(Object.prototype, 'bytes');
    }
    expect(observed).toBe(true);
  },
  30000
);
