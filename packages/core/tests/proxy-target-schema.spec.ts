import { describe, expect, test } from 'vitest';
import { parseUserConfig } from '../src/config/schema.js';

describe('proxy target schema regression', () => {
  test.each([
    'http://localhost:3000',
    {
      protocol: 'https:',
      host: 'localhost',
      port: 8443,
      pfx: Buffer.from('certificate'),
      passphrase: 'test'
    }
  ])('preserves supported target %s', (target) => {
    const config = parseUserConfig({
      server: { proxy: { '/api': { target } } }
    });
    expect(config.server?.proxy?.['/api'].target).toEqual(target);
  });
  test.each([123, true, null, [], undefined])(
    'rejects unsupported target %s',
    (target) => {
      expect(() =>
        parseUserConfig({ server: { proxy: { '/api': { target } } } } as never)
      ).toThrow();
    }
  );
});
