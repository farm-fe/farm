import { describe, expect, test, vi } from 'vitest';
import { normalizeResolverDeclarations } from './normalize-resolvers.js';

vi.mock('@vuetify/loader-shared', () => ({
  generateImports: (content: string) => {
    const imports: string[] = [];
    const source = content.replace(
      /(?:var|const) (\w+) *?= *?_resolveComponent\("([\w-.]+)"\);?/gm,
      (match, symbol: string, name: string) => {
        const component = name
          .split('-')
          .map((part) => part[0].toUpperCase() + part.slice(1))
          .join('');
        imports.push(`${component} as ${symbol}`);
        return ' '.repeat(match.length);
      }
    );
    return {
      code: `\nimport { ${imports.join(', ')} } from "vuetify/components";`,
      source
    };
  },
  includes: (values: unknown[], value: unknown) => values.includes(value),
  isObject: (value: unknown) =>
    value !== null && typeof value === 'object' && !Array.isArray(value),
  normalizePath: (value: string) => value,
  resolveVuetifyBase: () => '/node_modules/vuetify'
}));

const { default: vuetify } = await import('./index.js');

describe('normalizeResolverDeclarations', () => {
  test('preserves comments and nested declaration order', () => {
    const result = normalizeResolverDeclarations(
      'const first = _resolveComponent("v-app") /* comma , */, second = () => { const a = _resolveDirective("ripple"), b = value; return b; };'
    );
    expect(result).toMatch(/\/\* comma , \*\/;\n\s*const /);
    expect(result).toMatch(/_resolveDirective\("ripple"\);\n\s*const /);
    expect(result.indexOf('first =')).toBeLessThan(result.indexOf('second ='));
  });

  test('keeps strings, unrelated declarations and loop headers unchanged', () => {
    const content =
      'const message = \'const a = _resolveComponent("v-btn"), b = 1;\'; const left = 1, right = 2; for (const a = _resolveComponent("v-btn"), b = 1; b < 2;) { break; }';
    expect(normalizeResolverDeclarations(content)).toBe(content);
  });
});

function getImportPlugin(options?: Parameters<typeof vuetify>[0]) {
  const plugin = vuetify(options).find(
    (plugin) => plugin.name === 'js-plugin:vuetify:import'
  );

  if (!plugin) {
    throw new Error('Vuetify import plugin not found');
  }

  return plugin;
}

describe('farmPlugin', () => {
  test('omits the import transform when auto-import is disabled', () => {
    expect(
      vuetify({ autoImport: false }).some(
        (plugin) => plugin.name === 'js-plugin:vuetify:import'
      )
    ).toBe(false);
  });

  test('auto-imports every resolver in a combined Rust Vue declaration', async () => {
    const importPlugin = getImportPlugin();
    await importPlugin.configResolved?.({
      rustPlugins: [['/node_modules/@farmfe/plugin-vue/index.farm', '{}']]
    } as any);
    const result = await importPlugin.transform?.executor({
      content:
        'const _component_v_app = _resolveComponent("v-app"), _component_v_btn = _resolveComponent("v-btn");\nexport { _component_v_app, _component_v_btn };',
      query: [],
      resolvedPath: '/src/App.vue'
    } as any);
    expect(result?.content).toContain('VApp as _component_v_app');
    expect(result?.content).toContain('VBtn as _component_v_btn');
    expect(result?.content).not.toContain('_resolveComponent("v-btn")');
    expect(result?.content).not.toMatch(/^\s*,/m);
  });

  test('recognizes a Windows Rust Vue binary path', async () => {
    const importPlugin = getImportPlugin();
    await importPlugin.configResolved?.({
      rustPlugins: [
        [
          'C:\\project\\node_modules\\@farmfe\\plugin-vue-win32-x64-msvc\\index.farm',
          '{}'
        ]
      ]
    } as any);
  });

  test('skips nested JavaScript plugins when locating the Rust Vue plugin', async () => {
    const importPlugin = getImportPlugin();

    await importPlugin.configResolved?.({
      plugins: [[{ name: 'js-plugin:other' }]],
      rustPlugins: [
        ['/node_modules/@farmfe/plugin-vue-win32-x64-msvc/index.farm', '{}']
      ]
    } as any);

    const result = await importPlugin.transform?.executor({
      content: 'const _component_v_btn = _resolveComponent("v-btn");',
      query: [],
      resolvedPath: '/src/App.vue'
    } as any);

    expect(result?.content).toContain('vuetify/components');
  });

  test('accepts @farmfe/plugin-vue as the Vue SFC plugin', async () => {
    const importPlugin = getImportPlugin();

    await importPlugin.configResolved?.({
      vitePlugins: [],
      rustPlugins: [
        ['/node_modules/@farmfe/plugin-vue-linux-x64-gnu/index.farm', '{}']
      ]
    } as any);

    const result = await importPlugin.transform?.executor({
      content: 'const _component_v_btn = _resolveComponent("v-btn");',
      query: [],
      resolvedPath: '/src/App.vue'
    } as any);

    expect(result?.content).toContain('vuetify/components');
  });
});
