# Official Plugin Docs Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `website/docs/plugins/official-plugins` into alignment with the official plugin packages shipped by this repository.

**Architecture:** Treat the website docs as the source of user-facing plugin coverage, with `website/sidebars.js` defining navigation and `website/docs/plugins/official-plugins/overview.md` defining the public index. Add one focused MDX page per official plugin, sourced from each plugin package README or public entrypoint.

**Tech Stack:** Docusaurus 3, MDX, Farm official Rust plugins under `rust-plugins/*`, Farm official JS plugins under `js-plugins/*`.

---

## Investigation Summary

Current docs already cover these official plugins:

- Rust: `@farmfe/plugin-react`, `@farmfe/plugin-vue`, `@farmfe/plugin-sass`, `@farmfe/plugin-strip`, `@farmfe/plugin-dsv`, `@farmfe/plugin-yaml`, `@farmfe/plugin-virtual`, `@farmfe/plugin-react-components`
- JS: `@farmfe/js-plugin-postcss`, `@farmfe/js-plugin-less`, `@farmfe/js-plugin-sass`, `@farmfe/js-plugin-svgr`, `@farmfe/js-plugin-dts`

Official packages without current docs coverage:

- Rust: `@farmfe/plugin-auto-import`, `@farmfe/plugin-compress`, `@farmfe/plugin-dts`, `@farmfe/plugin-icons`, `@farmfe/plugin-image`, `@farmfe/plugin-mdx`, `@farmfe/plugin-modular-import`, `@farmfe/plugin-replace-dirname`, `@farmfe/plugin-svgr`, `@farmfe/plugin-tailwindcss`, `@farmfe/plugin-url`, `@farmfe/plugin-wasm`, `@farmfe/plugin-worker`
- JS: `@farmfe/js-plugin-babel`, `@farmfe/js-plugin-copy`, `@farmfe/js-plugin-electron`, `@farmfe/js-plugin-qiankun`, `@farmfe/js-plugin-react-compiler`, `@farmfe/js-plugin-tailwindcss`, `@farmfe/js-plugin-visualizer`, `@farmfe/js-plugin-vuetify`

## File Structure

- Modify `website/sidebars.js`: add official plugin docs pages to the plugin sidebar in the existing Rust/JS plugin categories.
- Modify `website/docs/plugins/official-plugins/overview.md`: link every documented official plugin from the overview.
- Create `website/docs/plugins/official-plugins/tailwindcss.mdx`: Rust TailwindCSS plugin docs from `rust-plugins/tailwindcss/index.d.ts` and package metadata.
- Create `website/docs/plugins/official-plugins/js-tailwindcss.mdx`: JS TailwindCSS plugin docs from `js-plugins/tailwindcss/src/index.ts`.
- Create `website/docs/plugins/official-plugins/js-visualizer.mdx`: JS Visualizer docs from `js-plugins/visualizer/README.md` and `src/types.ts`.
- Create `website/docs/plugins/official-plugins/js-electron.mdx`: Electron plugin docs from `js-plugins/electron/README.md` and `src/index.ts`.

## Task 1: First Official Plugin Coverage Slice

**Files:**
- Modify: `website/sidebars.js`
- Modify: `website/docs/plugins/official-plugins/overview.md`
- Create: `website/docs/plugins/official-plugins/tailwindcss.mdx`
- Create: `website/docs/plugins/official-plugins/js-tailwindcss.mdx`
- Create: `website/docs/plugins/official-plugins/js-visualizer.mdx`
- Create: `website/docs/plugins/official-plugins/js-electron.mdx`

- [ ] **Step 1: Add docs for `@farmfe/plugin-tailwindcss`**

Create `website/docs/plugins/official-plugins/tailwindcss.mdx` with install commands, Rust plugin registration, `content` and `config` option notes, and a minimal CSS example using `@import "tailwindcss";`.

- [ ] **Step 2: Add docs for `@farmfe/js-plugin-tailwindcss`**

Create `website/docs/plugins/official-plugins/js-tailwindcss.mdx` with install commands, JS plugin import usage, filter options, and a minimal CSS example using `@import "tailwindcss";`.

- [ ] **Step 3: Add docs for `@farmfe/js-plugin-visualizer`**

Create `website/docs/plugins/official-plugins/js-visualizer.mdx` with install commands, plugin usage, and `host`/`port` option documentation.

- [ ] **Step 4: Add docs for `@farmfe/js-plugin-electron`**

Create `website/docs/plugins/official-plugins/js-electron.mdx` with install commands, main/preload build configuration, and option types.

- [ ] **Step 5: Wire the new pages into navigation and overview**

Add the new Rust TailwindCSS page to the Rust Plugins sidebar category. Add the JS TailwindCSS, Visualizer, and Electron pages to the JS Plugins sidebar category. Update `overview.md` so these entries are clickable and include the Rust TailwindCSS plugin in the Rust list.

- [ ] **Step 6: Validate**

Run:

```bash
cd /tmp/workspace/farm-fe/farm/website
corepack pnpm build
```

Expected: Docusaurus build completes without broken MDX or sidebar references.

## Task 2: README-Backed Rust Plugin Coverage

**Files:**
- Create one page per package under `website/docs/plugins/official-plugins/`
- Modify `website/sidebars.js`
- Modify `website/docs/plugins/official-plugins/overview.md`

- [x] Add pages for README-backed Rust plugins: `svgr`, `wasm`, `worker`, `url`, `icons`, `image`, `compress`, and `modular-import`.
- [x] Use each package README as the source of installation, usage, options, and examples.
- [x] Wire every new page into the Rust Plugins sidebar category and overview.
- [ ] Validate with `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website`.

## Task 3: Source-Backed Remaining Plugin Coverage

**Files:**
- Create one page per remaining package under `website/docs/plugins/official-plugins/`
- Modify `website/sidebars.js`
- Modify `website/docs/plugins/official-plugins/overview.md`

- [ ] Add pages for remaining Rust plugins: `auto-import`, `dts`, `mdx`, and `replace-dirname`.
- [ ] Add pages for remaining JS plugins: `babel`, `copy`, `qiankun`, `react-compiler`, and `vuetify`.
- [ ] Source options from `index.d.ts`, `src/index.ts`, package README files, and examples.
- [ ] Validate with `corepack pnpm build` from `/tmp/workspace/farm-fe/farm/website`.
