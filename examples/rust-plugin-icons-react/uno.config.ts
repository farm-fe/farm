// uno.config.ts
import {
  defineConfig,
  presetIcons,
  presetUno,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'

export default defineConfig({
  content: {
    filesystem: [
      '**/*.{html,js,ts,jsx,tsx,vue,svelte,astro}',
    ],
  },
  presets: [
    presetIcons(),
    presetUno(),
  ],
  transformers: [transformerVariantGroup(), transformerDirectives()],
})
