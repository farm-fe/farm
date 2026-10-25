import { expect, test } from 'vitest';
import { getSortedPlugins } from '../src/plugin/index.js';
import type { JsPlugin } from '../src/plugin/type.js';

for (const asynchronous of [false, true]) {
  test(`metadata proxies stay bounded across 20,000 shared-context hooks (${asynchronous ? 'async' : 'sync'})`, async () => {
    const context = {
      stored: new Map<string, string>(),
      writes: 0,
      writeMetadata(key: string, value: string) {
        this.writes++;
        this.stored.set(key, value);
      },
      readMetadata(key: string) {
        const value = this.stored.get(key);
        return asynchronous ? Promise.resolve(value) : value;
      },
      readMetadataByScope() {
        const values = [...this.stored.values()];
        return asynchronous ? Promise.resolve(values) : values;
      }
    };
    const plugins: JsPlugin[] = ['one', 'two'].map((name) => ({
      name,
      transform: { filters: { moduleTypes: ['js'] }, executor: () => undefined }
    }));
    const sorted = getSortedPlugins(plugins);
    const invoke = (index: number) =>
      (sorted[index].transform as any).executor({}, context);
    invoke(0);
    const proxies = [
      context.writeMetadata,
      context.readMetadata,
      context.readMetadataByScope
    ];
    for (let index = 0; index < 20000; index++) invoke(index % 2);
    // Actual native-style receiver and JSON semantics must survive large graphs.
    (context.writeMetadata as any)('category', { stock: 12 });
    expect(context.writes).toBe(1);
    expect(context.stored.get('category')).toBe('{"stock":12}');
    expect(await context.readMetadata('category')).toEqual({ stock: 12 });
    expect(await context.readMetadataByScope()).toEqual([{ stock: 12 }]);
    expect([
      context.writeMetadata,
      context.readMetadata,
      context.readMetadataByScope
    ]).toEqual(proxies);
  });
}
