import { describe, expect, it } from 'vitest';
import { parseUserConfig } from '../src/config/schema.js';

describe('preview host configuration', () => {
  it.each(['127.0.0.1', '0.0.0.0', '192.168.0.7'])(
    'accepts IPv4 host %s consistently with the dev server',
    (host) => {
      const config = parseUserConfig({ server: { host, preview: { host } } });
      expect(config.server?.host).toBe(host);
      expect(config.server?.preview?.host).toBe(host);
    }
  );

  it.each(['localhost', true, false])(
    'preserves supported preview host %s',
    (host) => {
      expect(
        parseUserConfig({ server: { preview: { host } } }).server?.preview?.host
      ).toBe(host);
    }
  );

  it('rejects a two-octet address before starting the preview server', () => {
    expect(() =>
      parseUserConfig({ server: { preview: { host: '127.0' } } })
    ).toThrow(/server.preview.host/);
  });
});
