<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <span>English</span> |
    <a href="./README-zh-CN.md">简体中文</a>  
</div>

---

# Electron Plugin for Farm

Support development Electron App using Farm.

## Getting Started

To begin, you'll need to `@farmfe/js-plugin-electron`:

```bash
npm install @farmfe/js-plugin-electron --save-dev
```

or

```bash
yarn add -D @farmfe/js-plugin-electron
```

or

```bash
pnpm add -D @farmfe/js-plugin-electron
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineConfig } from 'farm'
import electron from '@farmfe/js-plugin-electron'

import { defineConfig } from 'farm'
import electron from './farm-plugin-electron'

export default defineConfig({
  plugins: [
    electron({
      main: {
        input: 'electron/main.ts',
      },
      preload: {
        input: 'electron/preload.ts',
      },
    }),
  ],
})
```

## Options

Type:

```ts
import type { UserConfig } from 'farm'

export interface BuildOptions {
  /**
   * Shortcut of `compilation.input`
   */
  input: string | Record<string, string>
  farm?: UserConfig
}

export interface ElectronPluginOptions {
  main: BuildOptions
  preload?: BuildOptions
}
```
