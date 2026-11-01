// Directory-level validation contract. Changes to this file are global inputs.
// npm dependencies (including dev/peer/optional) are validation edges. Cargo
// native jobs remain full until their execution receipts support crate subsets.
export const directoryImpactRules = Object.freeze({
  version: 1,
  global: Object.freeze([
    '.github/', '.cargo/', 'configs/', 'scripts/', 'patches/', 'crates/', 'rust-plugins/',
    'Cargo.toml', 'Cargo.lock', 'rust-toolchain', 'rust-toolchain.toml',
    'package.json', 'pnpm-lock.yaml', 'pnpm-workspace.yaml', 'tsconfig.json',
    'vitest.config.ts', 'biome.json', '.npmrc', 'tsconfig.base.json',
    'packages/core/', 'packages/cli/', 'packages/runtime/',
    'packages/runtime-plugin-hmr/', 'packages/runtime-plugin-import-meta/',
    'packages/plugin-tools/', 'packages/utils/', 'e2e/', 'examples/ssr-utils/'
  ]),
  dependencies: Object.freeze([
    // PostCSS's ordinary TS spec resolves UnoCSS via this Example manifest.
    { consumer: 'js-plugins/postcss', dependencies: ['examples/vite-adapter-vue', 'js-plugins/tailwindcss'] },
    // Generated JS plugin templates consume DTS outside the root manifest.
    { consumer: 'packages/create-farm-plugin', dependencies: ['js-plugins/dts'] },
    // The shared Example build recipe prepares Docs and its theme together.
    { consumer: 'packages/docs', dependencies: ['packages/docs-theme-default'] },
    { consumer: 'packages/docs-theme-default', dependencies: ['packages/docs'] }
  ])
});
