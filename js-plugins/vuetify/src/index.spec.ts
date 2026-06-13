import { describe, expect, test, vi } from 'vitest';

vi.mock('@vuetify/loader-shared', () => ({
  generateImports: (content: string) => ({
    code: '\nimport { VBtn } from "vuetify/components";',
    source: content
  }),
  includes: (values: unknown[], value: unknown) => values.includes(value),
  isObject: (value: unknown) =>
    value !== null && typeof value === 'object' && !Array.isArray(value),
  normalizePath: (value: string) => value,
  resolveVuetifyBase: () => '/node_modules/vuetify'
}));

const { default: vuetify } = await import('./index.js');

function getImportPlugin() {
  const plugin = vuetify().find(
    (plugin) => plugin.name === 'js-plugin:vuetify:import'
  );

  if (!plugin) {
    throw new Error('Vuetify import plugin not found');
  }

  return plugin;
}

describe('farmPlugin', () => {
  test('accepts @farmfe/plugin-vue as the Vue SFC plugin', async () => {
    const importPlugin = getImportPlugin();

    await importPlugin.configResolved?.({
      vitePlugins: [],
      rustPlugins: [
        ['/node_modules/@farmfe/plugin-vue-linux-x64-gnu/index.farm', '{}']
      ]
    } as any);

    const result = await importPlugin.transform?.executor({
      content: '<template><v-btn /></template>',
      query: [],
      resolvedPath: '/src/App.vue'
    } as any);

    expect(result?.content).toContain('vuetify/components');
  });
});
