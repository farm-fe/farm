---
name: building-farm-from-source
description: Use when building Farm locally from source, native bindings or Rust plugin artifacts are missing, or an example/E2E needs local core and plugin artifacts.
license: MIT
compatibility: Requires Node, pnpm, cargo, npm scripts, and optional Playwright browsers for E2E.
metadata:
  author: farm
  version: "1.0"
---

# Building Farm From Source

## Overview

Farm examples depend on generated JS packages and native `.node` / `.farm`
artifacts. Build missing prerequisites from the bottom up before running example
builds or E2E.

## When to Use

- `napi: not found`, missing `binding.cjs`, or missing `farm.*.node`.
- `Cannot find module '@farmfe/plugin-*-linux-x64-gnu'`.
- `packages/cli/dist/index.js` or package `dist/` files are missing.
- Running `examples/<name>` or `pnpm run test-e2e -- --example <name>` locally.

## Command Order

Run from `/home/runner/work/farm/farm` unless noted:

1. Install dependencies if CLIs are missing:
   ```bash
   ELECTRON_SKIP_BINARY_DOWNLOAD=1 PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 corepack pnpm install
   ```
2. Build core native binding first:
   ```bash
   cd /home/runner/work/farm/farm/packages/core && npm run build:rs
   ```
3. Build JS package prerequisites:
   ```bash
   cd /home/runner/work/farm/farm/packages/utils && npx tsc -p tsconfig.json --outDir dist
   cd /home/runner/work/farm/farm
   corepack pnpm --filter @farmfe/runtime run build
   corepack pnpm --filter @farmfe/runtime-plugin-hmr run build
   corepack pnpm --filter @farmfe/runtime-plugin-import-meta run build
   corepack pnpm --filter @farmfe/plugin-tools run build
   ```
4. Build Rust plugin artifacts required by core/examples:
   ```bash
   cd /home/runner/work/farm/farm/rust-plugins/replace-dirname && npm run build
   cd /home/runner/work/farm/farm/rust-plugins/react && npm run build
   cd /home/runner/work/farm/farm/rust-plugins/tailwindcss && npm run build
   ```
5. Build core JS and CLI:
   ```bash
   cd /home/runner/work/farm/farm
   corepack pnpm --filter @farmfe/core run build
   corepack pnpm --filter @farmfe/cli run build
   ```
6. Build and test the affected example:
   ```bash
   cd /home/runner/work/farm/farm/examples/tailwindcss-rust-plugin && npm run build
   cd /home/runner/work/farm/farm
   corepack pnpm exec playwright install chromium
   corepack pnpm run test-e2e -- --example tailwindcss-rust-plugin
   ```

## Debugging Missing Prerequisites

| Symptom | Build first |
|---------|-------------|
| `napi: not found` | `corepack pnpm install` |
| Missing `@farmfe/utils/colors` | `cd packages/utils && npx tsc -p tsconfig.json --outDir dist` |
| Missing `@farmfe/plugin-replace-dirname-*` | `cd rust-plugins/replace-dirname && npm run build` |
| Missing `@farmfe/plugin-react-*` | `cd rust-plugins/react && npm run build` |
| Missing `@farmfe/plugin-tailwindcss-*` | `cd rust-plugins/tailwindcss && npm run build` |
| Missing `packages/cli/dist/index.js` | `corepack pnpm --filter @farmfe/cli run build` |
| Playwright executable missing | `corepack pnpm exec playwright install chromium` |

## Notes

- For build verification, prefer package `npm run build` scripts over direct
  Cargo commands.
- Rust plugin builds copy local artifacts into `rust-plugins/*/npm/<abi>/`.
- If `pnpm install` cannot download Electron or Playwright binaries, skip them
  during install and install Playwright Chromium only when E2E is needed.
