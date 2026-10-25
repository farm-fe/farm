import path from 'node:path';
import { expect, test, vi } from 'vitest';

vi.mock('@farmfe/core', () => ({
  Resolver: class {
    resolve() {
      return false;
    }
  },
  logger: {}
}));

import tailwindcss from '../src/index.js';

function required<T>(value: T | undefined): T {
  if (value === undefined) throw new Error('Required plugin hook is missing');
  return value;
}

test('new lazy source paths rebuild the previously generated Tailwind root exactly once', async () => {
  const [scan, generate] = tailwindcss();
  expect(generate.updateModules?.executor).toBeTypeOf('function');
  await scan.configResolved?.({
    root: process.cwd(),
    compilation: { root: process.cwd(), sourcemap: false }
  } as any);
  const root = path.resolve('tailwind-lazy-regression.css');
  const context = {
    addWatchFile: vi.fn(),
    readMetadata: () => ['grid', 'bg-rose-200'],
    writeMetadata: vi.fn(),
    readMetadataByScope: () => [['grid']]
  };
  await required(scan.transform).executor(
    {
      resolvedPath: path.resolve('fixture.tsx'),
      content: '<div className="grid" />'
    } as any,
    context as any
  );
  const result = await required(generate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    },
    context as any
  );
  expect(result?.content).toContain('display');
  const source = path.resolve(
    'late-page.tsx.farm_dynamic_import_virtual_module'
  );
  const paths: any = [[source, 'updated']];
  const updated = await required(generate.updateModules).executor(
    { paths },
    context as any
  );
  expect(updated).toEqual([...paths, [root, 'updated']]);
  expect(
    await required(generate.updateModules).executor(
      { paths: [...paths, [root, 'updated']] },
      context as any
    )
  ).toEqual([...paths, [root, 'updated']]);
  expect(
    await required(generate.updateModules).executor(
      { paths: [[root, 'removed']] },
      context as any
    )
  ).toBeUndefined();
});

test('fresh plugin instance restores generated roots before cached lazy source arrives', async () => {
  const config = {
    root: process.cwd(),
    compilation: { root: process.cwd(), sourcemap: false }
  } as any;
  const [coldScan, coldGenerate] = tailwindcss();
  await coldScan.configResolved?.(config);
  const root = path.resolve('tailwind-warm-lazy-regression.css');
  const context = {
    addWatchFile: vi.fn(),
    readMetadata: () => ['grid', 'bg-rose-200'],
    writeMetadata: vi.fn(),
    readMetadataByScope: () => [['grid']]
  };
  await required(coldScan.transform).executor(
    {
      resolvedPath: path.resolve('fixture.tsx'),
      content: '<div className="grid" />'
    } as any,
    context as any
  );
  await required(coldGenerate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    },
    context as any
  );
  const bytes =
    (await coldGenerate.writePluginCache?.executor(
      undefined,
      context as any
    )) ?? [];

  // Warm modules can be restored without invoking transform or freezeModule.
  const [warmScan, warmGenerate] = tailwindcss();
  await warmScan.configResolved?.(config);
  await warmGenerate.pluginCacheLoaded?.executor(bytes, context as any);
  await required(warmScan.pluginCacheLoaded).executor(
    await required(coldScan.writePluginCache).executor(
      undefined,
      context as any
    ),
    context as any
  );
  const paths: any = [
    [
      path.resolve('late-warm.tsx.farm_dynamic_import_virtual_module'),
      'updated'
    ]
  ];
  expect(
    await required(warmGenerate.updateModules).executor(
      { paths },
      context as any
    )
  ).toEqual([...paths, [root, 'updated']]);
  expect(
    await required(warmGenerate.updateModules).executor(
      { paths: [[root, 'removed']] },
      context as any
    )
  ).toBeUndefined();
  const afterRemoval = await required(warmGenerate.writePluginCache).executor(
    undefined,
    context as any
  );
  expect(JSON.parse(Buffer.from(afterRemoval).toString()).roots).toEqual([]);
});

test('restored candidate modules remain watch dependencies when the root regenerates', async () => {
  const config = {
    root: process.cwd(),
    compilation: { root: process.cwd(), sourcemap: false }
  } as any;
  const [scan, generate] = tailwindcss();
  await scan.configResolved?.(config);
  const root = path.resolve('tailwind-restored-watch.css');
  const source = path.resolve('previous-page.tsx');
  const context = {
    addWatchFile: vi.fn(),
    readMetadata: () => ['grid', 'bg-rose-200'],
    writeMetadata: vi.fn(),

    readMetadataByScope: () => [['grid']]
  };
  await required(scan.transform).executor(
    { resolvedPath: source, content: '<div className="grid" />' } as any,
    context as any
  );
  await required(generate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    },
    context as any
  );
  const scanBytes = await required(scan.writePluginCache).executor(
    undefined,
    context as any
  );
  const bytes = await required(generate.writePluginCache).executor(
    undefined,
    context as any
  );
  const [warmScan, warmGenerate] = tailwindcss();
  await warmScan.configResolved?.(config);
  await required(warmGenerate.pluginCacheLoaded).executor(
    bytes,
    context as any
  );
  await required(warmScan.pluginCacheLoaded).executor(
    scanBytes,
    context as any
  );
  context.addWatchFile.mockClear();
  const result = await required(warmGenerate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    },
    context as any
  );
  expect(result?.content).toContain('display');
  expect(context.addWatchFile).toHaveBeenCalledWith(root, source);
});

test('malformed plugin state is rejected before replacing active roots', async () => {
  const [scan, generate] = tailwindcss();
  await scan.configResolved?.({
    root: process.cwd(),
    compilation: { root: process.cwd() }
  } as any);
  const context = {
    addWatchFile: vi.fn(),
    readMetadata: () => ['grid', 'bg-rose-200'],
    writeMetadata: vi.fn(),
    readMetadataByScope: () => [['grid']]
  };
  const root = path.resolve('tailwind-invalid-cache.css');
  await required(generate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    },
    context as any
  );
  const before = await required(generate.writePluginCache).executor(
    undefined,
    context as any
  );
  for (const state of [
    { version: 2, roots: [], candidateModules: [] },
    { version: 1, roots: [42], candidateModules: [] },
    { version: 1, roots: null }
  ]) {
    expect(() =>
      required(generate.pluginCacheLoaded).executor(
        Array.from(Buffer.from(JSON.stringify(state))),
        context as any
      )
    ).toThrow('Invalid Tailwind plugin cache');
    expect(
      await required(generate.writePluginCache).executor(
        undefined,
        context as any
      )
    ).toEqual(before);
  }
});

test.each([false, true])(
  'ordinary CSS does not watch scanned Tailwind candidates (warm=%s)',
  async (warm) => {
    const config = {
      root: process.cwd(),
      compilation: { root: process.cwd(), sourcemap: false }
    } as any;
    const [scan, generate] = tailwindcss();
    await scan.configResolved?.(config);
    const source = path.resolve('commerce-component.tsx');
    const stylesheet = path.resolve('ordinary-vendor.css');
    const context = {
      addWatchFile: vi.fn(),
      readMetadata: () => [],
      writeMetadata: vi.fn(),
      readMetadataByScope: () => [['bg-rose-200']]
    };
    if (warm) {
      await required(scan.pluginCacheLoaded).executor(
        Array.from(Buffer.from(JSON.stringify([[source, false]]))),
        context as any
      );
      await required(generate.pluginCacheLoaded).executor(
        Array.from(
          Buffer.from(
            JSON.stringify({
              version: 1,
              roots: [],
              candidateModules: [source]
            })
          )
        ),
        context as any
      );
    } else {
      await required(scan.transform).executor(
        {
          resolvedPath: source,
          content: '<div className="bg-rose-200" />'
        } as any,
        context as any
      );
    }
    const content = '.vendor-button { color: red; }';
    await required(scan.transform).executor(
      {
        resolvedPath: path.resolve('fixture.tsx'),
        content: '<div className="grid" />'
      } as any,
      context as any
    );
    const result = await required(generate.freezeModule).executor(
      {
        moduleId: stylesheet,
        moduleType: 'css',
        content,
        sourceMapChain: [],
        resolvedDeps: []
      },
      context as any
    );
    expect(result?.content).toBe(content);
    expect(context.addWatchFile).not.toHaveBeenCalledWith(stylesheet, source);
    const state = await required(generate.writePluginCache).executor(
      undefined,
      context as any
    );
    expect(JSON.parse(Buffer.from(state).toString()).roots).toEqual([]);
  }
);

test('unrelated/vendor updates do not rebuild roots and removed roots stay removed', async () => {
  const [scan, generate] = tailwindcss();
  await scan.configResolved?.({
    root: process.cwd(),
    compilation: { sourcemap: false }
  } as any);
  const root = path.resolve('scoped-removal.css');
  const context = {
    addWatchFile: vi.fn(),
    readMetadata: () => ['grid'],
    writeMetadata: vi.fn()
  };
  await required(scan.transform).executor(
    {
      resolvedPath: path.resolve('scope.tsx'),
      content: '<div className="grid" />'
    } as any,
    context as any
  );
  await required(generate.freezeModule).executor(
    {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    } as any,
    context as any
  );
  for (const id of [
    'ordinary.css',
    'asset.png',
    'node_modules/vendor/index.js'
  ]) {
    expect(
      await required(generate.updateModules).executor(
        { paths: [[path.resolve(id), 'updated']] },
        context as any
      )
    ).toBeUndefined();
  }
  const paths: any = [
    [root, 'removed'],
    [path.resolve('page.tsx'), 'updated']
  ];
  expect(
    await required(generate.updateModules).executor({ paths }, context as any)
  ).toEqual(paths);
  const next: any = [[path.resolve('next.tsx'), 'updated']];
  expect(
    await required(generate.updateModules).executor(
      { paths: next },
      context as any
    )
  ).toEqual(next);
  expect(
    JSON.parse(
      Buffer.from(
        await required(generate.writePluginCache).executor(
          undefined,
          context as any
        )
      ).toString()
    ).roots
  ).toEqual([]);
});

test('warm scan preserves physical/query identity and prunes deleted physical sources', async () => {
  const fs = await import('node:fs/promises'),
    os = await import('node:os');
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), 'farm-tailwind-scope-')
  );
  try {
    const source = path.join(directory, 'page.tsx'),
      query = source + '?component',
      root = path.join(directory, 'style.css');
    await fs.writeFile(source, '<div className="grid" />');
    const config = {
      root: directory,
      compilation: { sourcemap: false }
    } as any;
    const [scan, generate] = tailwindcss();
    await scan.configResolved?.(config);
    const context = {
      addWatchFile: vi.fn(),
      readMetadata: vi.fn(() => ['grid']),
      writeMetadata: vi.fn()
    };
    await required(scan.transform).executor(
      { resolvedPath: query, content: '<div className="grid" />' } as any,
      context as any
    );
    const css = {
      moduleId: root,
      moduleType: 'css',
      content: '@tailwind utilities;',
      sourceMapChain: [],
      resolvedDeps: []
    } as any;
    await required(generate.freezeModule).executor(css, context as any);
    const scanBytes = await required(scan.writePluginCache).executor(
      undefined,
      context as any
    );
    const generateBytes = await required(generate.writePluginCache).executor(
      undefined,
      context as any
    );
    expect(JSON.parse(Buffer.from(scanBytes).toString())).toEqual([
      [query, true]
    ]);
    const [warmScan, warmGenerate] = tailwindcss();
    await warmScan.configResolved?.(config);
    await required(warmScan.pluginCacheLoaded).executor(
      scanBytes,
      context as any
    );
    await required(warmGenerate.pluginCacheLoaded).executor(
      generateBytes,
      context as any
    );
    context.addWatchFile.mockClear();
    context.readMetadata.mockClear();
    await required(warmGenerate.freezeModule).executor(css, context as any);
    expect(context.addWatchFile).toHaveBeenCalledWith(root, source);
    expect(context.addWatchFile).not.toHaveBeenCalledWith(root, query);
    expect(context.readMetadata).toHaveBeenCalledWith(query, {
      refer: [query],
      scope: ['tailwindcss:candidateScope']
    });
    await fs.unlink(source);
    context.addWatchFile.mockClear();
    await required(warmGenerate.freezeModule).executor(css, context as any);
    expect(context.addWatchFile).not.toHaveBeenCalledWith(root, source);
    expect(
      JSON.parse(
        Buffer.from(
          await required(warmScan.writePluginCache).executor(
            undefined,
            context as any
          )
        ).toString()
      )
    ).toEqual([]);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
