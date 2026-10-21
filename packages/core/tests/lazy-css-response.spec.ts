import { describe, expect, test, vi } from 'vitest';
import { lazyCompilationMiddleware } from '../src/server/middlewares/lazyCompilation.js';

describe('lazy compilation CSS registration response', () => {
  async function request(node = false) {
    const update = vi.fn(async () => ({
      dynamicResourcesMap: null,
      changed: ['styles.css'],
      immutableModules: '',
      mutableModules: 'globalThis.__farmLazyCssRegistration = 17;'
    }));
    const app = {
      config: {
        logger: { info: vi.fn(), formatTime: () => '1ms' },
        server: {}
      },
      compiler: {
        config: { root: process.cwd() },
        transformModulePath: (_root: string, file: string) => file,
        update,
        writeResourcesToDisk: vi.fn()
      }
    };
    let body = '';
    await lazyCompilationMiddleware(app as any)(
      {
        url: `/__lazy_compile?paths=page.tsx${node ? '&node=true' : ''}`,
        headers: {}
      } as any,
      {
        setHeader: vi.fn(),
        end: (value: string) => {
          body = value;
        },
        writableEnded: false
      } as any,
      vi.fn()
    );
    return { body, update };
  }
  test('browser response registers only the compiler-provided CSS update code and identifies changed modules', async () => {
    const { body, update } = await request();
    expect(body).toContain('globalThis.__farmLazyCssRegistration = 17;');
    expect(body).toContain('"changedModules":["styles.css"]');
    expect(body.indexOf('__farmLazyCssRegistration')).toBeLessThan(
      body.indexOf('export default')
    );
    expect(update.mock.calls[0][3]).toBe(false);
  });
  test('Node lazy response stays JSON and does not execute browser registration code', async () => {
    const { body } = await request(true);
    expect(() => JSON.parse(body)).not.toThrow();
    expect(body).not.toContain('__farmLazyCssRegistration');
  });
});
