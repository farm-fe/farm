import { expect, test } from 'vitest';
import {
  DEFAULT_DEV_SERVER_OPTIONS,
  DEFAULT_HMR_OPTIONS
} from '../config/index.js';
import merge from './merge.js';

test('merge - base', () => {
  const target = { a: 1, b: 2 };
  const source = { b: 3, c: 4 };
  const result = merge(target, source);

  expect(result).toEqual({ a: 1, b: 3, c: 4 });
});

test('merge - nested', () => {
  const hmr = merge(
    {},
    DEFAULT_HMR_OPTIONS,
    { host: undefined, port: undefined },
    {}
  );
  expect(hmr).toEqual({
    ...DEFAULT_HMR_OPTIONS
  });

  const server = merge({}, DEFAULT_DEV_SERVER_OPTIONS, undefined, {
    hmr: DEFAULT_HMR_OPTIONS,
    https: undefined
  });

  expect(server).toEqual({
    ...DEFAULT_DEV_SERVER_OPTIONS,
    hmr,
    https: undefined
  });
});

test('merge - remove duplication', () => {
  const res = merge({ css: [1, 'a', { a: 1 }] }, { css: [2, 'a', { b: 2 }] });
  expect(res).toEqual({ css: [1, 'a', { a: 1 }, 2, { b: 2 }] });

  const res2 = merge(
    { css: [{ b: 1 }, { a: 1 }] },
    { css: [{ a: 1 }, { b: 2 }] }
  );
  expect(res2).toEqual({ css: [{ b: 1 }, { a: 1 }, { b: 2 }] });
});

test('merge - preserve custom object', () => {
  class SuperSpecial {
    special: string;

    constructor() {
      this.special = 'oh yeah man totally';
    }
  }

  const instantiatedSpecialObject = new SuperSpecial();

  const target: any = {
    someProperty: {
      cool: 'oh for sure'
    }
  };

  const source: any = {
    someProperty: {
      ins: instantiatedSpecialObject
    }
  };

  const defaultOutput = merge(target, source);
  expect(defaultOutput.someProperty.cool).toBe('oh for sure');
  expect(defaultOutput.someProperty.special).toBe(undefined);
  expect(defaultOutput.someProperty.ins instanceof SuperSpecial).toBe(true);
  expect(defaultOutput.someProperty.ins.special).toBe('oh yeah man totally');
});
