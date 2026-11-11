---
title: Module Federation (experimental)
description: Configure native Farm Module Federation, shared React dependencies, and cross-application loading.
---

# Module Federation (experimental)

:::warning Local experimental implementation
`@farmfe/plugin-module-federation` is a private, unpublished workspace package. There is no released installation version. Use a source checkout containing the plugin, its matching core changes, and the example below; an older published Farm core does not include the async CommonJS fix. This page documents the local verified implementation, not a promise of complete Federation compatibility.
:::

## How it works

The Rust compiler plugin resolves remote/shared requests into Farm's module graph and generates the container. A thin adapter connects the Farm module system to official Module Federation runtime 2.9.2, which handles loading and shared-version selection. Farm still owns business module registration, caching, execution, dynamic JS chunks and CSS. The adapter is loaded through the module graph; it does not depend on an asynchronous bootstrap hook.

The default container is a classic script exposing standard `get`/`init` on a named global. It does not require native ESM remote loading. Optional ESM containers retain the same standard interface, so other bundlers do not need to understand Farm module IDs.

## Try the verified example

Build the matching source packages following [Contributing](../contribution.mdx). Then run:

```bash
# From the repository root, with the matching core JS/native packages built:
pnpm install --frozen-lockfile
node rust-plugins/module-federation/scripts/build-native.mjs
pnpm exec playwright install chromium
pnpm run test-e2e -- --example module-federation-native-probe
```

The native build command defaults to the local `ci` profile. It accepts `--profile`, `--target`, an optional matching `--abi`, and Cargo options such as `--target-dir`. Output is staged at `npm/<abi>/index.farm` for `darwin-arm64`, `linux-x64-gnu`, or `win32-x64-msvc`; use the same checkout and pinned Rust toolchain as the core, with compatible target/ABI settings. Local `ci` verification uses a `ci`-built core and plugin. Formal CI follows the existing core `release-publish` and plugin `release` recipes, so identical profile names are not required. Explicit targets still require the appropriate Rust toolchain and linker. Native execution has been verified locally on macOS arm64; Linux and Windows need their own acceptance run. In a cows-managed workspace, keep pnpm, Cargo and browser caches inside its overlay.

The example package is `examples/module-federation-native-probe`. Its `config.mjs` defines host 4100, remote A 4101 at `/deploy/a/`, and remote B 4102 at `/deploy/b/`. Its README describes native prerequisites, reserved ports, logs and cleanup. After building prerequisites, `pnpm --filter @farmfe-examples/module-federation-native-probe start` runs the three dev apps. Open `http://127.0.0.1:4100`.

The repository E2E command builds and serves independent production origins, starts real dev servers, renders remote React hooks components, checks singleton identity and resources, tests failure/reload recovery, and runs the Rspack/Webpack matrix. It uses timeouts and cleans up owned processes; no manually started service is needed. Windows cleanup targets only a still-running owned child and its descendants, with IPC for graceful suite shutdown. If that child has already exited or crashed, orphaned descendants may remain; recovery of that case is not guaranteed. Install Chromium before running it, or point `PLAYWRIGHT_BROWSERS_PATH` at an existing matching installation.

## Minimal host and remote

The following files belong in one local workspace example with dependencies on `@farmfe/core`, `@farmfe/cli`, `@farmfe/plugin-module-federation` via `workspace:*`, and React/ReactDOM **18.3.1**. The native plugin supplies its pinned official runtime dependency. The shared file sits next to `host/` and `remote/`. These examples use JavaScript's JSX runtime directly, so no JSX transform plugin is required.

```js title="shared.mjs"
import { fileURLToPath } from 'node:url';

export const shared = Object.fromEntries(
  ['react', 'react/jsx-runtime'].map((key) => [key, {
    version: '18.3.1',
    requiredVersion: '^18.3.1',
    singleton: true,
    strictVersion: true,
    fallback: fileURLToPath(import.meta.resolve(key)),
  }]),
);
```

```js title="remote/farm.config.mjs"
import { fileURLToPath } from 'node:url';
import federation from '@farmfe/plugin-module-federation';
import { shared } from '../shared.mjs';

export default {
  root: fileURLToPath(new URL('.', import.meta.url)),
  server: { host: '127.0.0.1', port: 4101, strictPort: true,
    origin: 'http://127.0.0.1:4101', cors: true, hmr: false },
  compilation: {
    input: { remoteEntry: 'farm-mf-native:container' },
    output: { targetEnv: 'browser-esnext', publicPath: '/deploy/a/' },
    sourcemap: false,
    lazyCompilation: false,
    css: { transformToScript: false },
  },
  plugins: [federation({
    name: 'providerA',
    globalName: 'remoteA',
    publicPath: 'http://127.0.0.1:4101/deploy/a/',
    format: 'global',
    filename: 'remoteEntry.js',
    exposes: { './Button': './Button.js' },
    shared,
  })],
};
```

```js title="remote/Button.js"
import react from 'react';
import { jsx } from 'react/jsx-runtime';
import './button.css';

export const loadLabel = async () => (await import('./lazy.js')).label;

export function Counter() {
  const [label, setLabel] = react.useState('Remote');
  return jsx('button', {
    className: 'remote-button',
    onClick: async () => setLabel(await loadLabel()),
    children: label,
  });
}
```

```css title="remote/button.css"
.remote-button { color: rgb(17, 85, 153); }
```

```js title="remote/lazy.js"
export const label = 'Loaded from the remote chunk';
```

```js title="host/farm.config.mjs"
import { fileURLToPath } from 'node:url';
import federation from '@farmfe/plugin-module-federation';
import { shared } from '../shared.mjs';

export default {
  root: fileURLToPath(new URL('.', import.meta.url)),
  server: { host: '127.0.0.1', port: 4100, strictPort: true, hmr: false },
  compilation: {
    input: { index: './index.html' },
    output: { targetEnv: 'browser-esnext' },
    sourcemap: false,
    lazyCompilation: false,
  },
  plugins: [federation({
    name: 'consumer',
    remotes: {
      remote: { name: 'providerA', globalName: 'remoteA', format: 'global',
        entry: 'http://127.0.0.1:4101/deploy/a/remoteEntry.js' },
    },
    shared,
  })],
};
```

```html title="host/index.html"
<!doctype html>
<div id="app"></div>
<script type="module" src="./main.js"></script>
```

```js title="host/main.js"
import { createRoot } from 'react-dom/client';
import { jsx } from 'react/jsx-runtime';
import { Counter } from 'remote/Button';

createRoot(document.querySelector('#app')).render(jsx(Counter, {}));

const remote = await import('remote/Button');
console.log(await remote.loadLabel());
```

Run `pnpm exec farm start --config remote/farm.config.mjs` and `pnpm exec farm start --config host/farm.config.mjs` from that example's root in separate terminals. The remote must be reachable before the host can finish its static remote import. For production, run the corresponding `farm build --config ...` commands and deploy each application's `dist` to its configured origin/subpath. Enable appropriate CORS and MIME types, especially for ESM entries. The automated example uses explicit loopback binding and controlled static servers.

## Configuration reference

| Option | Default | Meaning |
| --- | --- | --- |
| `name` | Required | Unique application name; a JavaScript identifier. Farm namespace becomes `farm_mf_<name>`. |
| `hot` | `false` | Opt-in development-only Farm-to-Farm HMR with explicit consumer acceptance. |
| `globalName` | `name` | Container global; must be a JavaScript identifier. |
| `format` | `'global'` | Producer container format: `'global'` or `'esm'`. |
| `filename` | `'remoteEntry.js'` | Plain `.js` filename, without directory separators. |
| `publicPath` | `compilation.output.publicPath` | Absolute HTTP(S) base for remote JS/CSS resources, including the deployment subpath and trailing slash. Required to resolve to an absolute URL when using exposes. |
| `exposes` | `{}` | Map of `./`-prefixed public keys to source paths resolved against the application root. |
| `remotes` | `{}` | Map of import aliases to remote descriptors (below). |
| `shared` | `{}` | Map of exact package requests to shared descriptors (below). |

A remote descriptor requires `name` (the producer identity) and `entry` (an HTTP(S) entry URL). Its `globalName` defaults to its `name`; `format` defaults to `'global'`. The object key is the import alias: `remote` maps `remote/Button` to the producer's `./Button`. Use distinct aliases/names, avoiding prefix collisions such as alias `remote` with another remote named `remoteB` in the official runtime.

A shared descriptor requires `version`, `requiredVersion`, and `fallback` (an absolute local module path). `singleton` and `strictVersion` both default to `false`. There is no automatic dependency/version inference, wildcard subpath matching, or `fallback: false`. Configure `react/jsx-runtime` separately from `react`. The wrapper supplies `runtimePath`; applications should not set it. Unknown Rust option keys are rejected.

The native plugin currently creates a dedicated container build when `exposes` is nonempty: it replaces the input map and uses `filename` for the entry. Set the virtual input shown above so the JavaScript configuration normalizer does not expect an `index.html`. Use a separate build if the remote also needs an ordinary application page. This is a browser-only plugin.

## Shared React and initialization

Configure the same React root and JSX subpath in host and remote. The example's host ReactDOM renders the remote component; `singleton: true` prevents a second selected React instance, and `strictVersion: true` rejects an incompatible loaded provider. `requiredVersion` is negotiated by the official runtime. The `fallback` points to the real local package used when no suitable provider is supplied; it is not merely configuration registration.

The tested strategy is **loaded-first** with one share scope. The container adopts the host scope before exposing modules. Repeated initialization with the same scope shares a promise; a different scope is rejected. Static remote imports and shared proxies are async graph nodes, so dependent application code waits before execution. Import-namespace wrapper objects may differ between Farm runtimes; compare actual React default exports or `jsx` functions when checking identity.

## CSS, chunks and deployment URLs

Remote CSS and lazy JS are loaded by the remote's isolated Farm runtime, not relative to the host. Keep `name` unique across applications, including independent deployments. Keep `publicPath` aligned with the deployed remote resources. In dev, the relative `compilation.output.publicPath` routes the server, `server.origin` gives its origin, and the plugin's absolute `publicPath` initializes remote resource loading. In production the same absolute MF base must serve all emitted chunks and CSS, not just `remoteEntry.js`.

Federation is opt-in, but the supporting compiler changes are shared with non-Federation builds: async CommonJS evaluation preserves source order, and injected CSS style identities and cache keys include the runtime namespace. Lazy compilation must match styles in its own namespace while invalidating the original module IDs. The public compiler/plugin interfaces and HMR runtime APIs are unchanged. The Federation browser matrix below disables `lazyCompilation`; it is not evidence for every Federation/lazy-compilation combination.

Without `hot: true`, remote edits still use full-page refresh. Opt-in cross-application HMR is described below. A failed static remote load prevents bootstrap. Restoring the endpoint and reloading the document is the tested recovery boundary; same-page automatic retry is not promised.

## Opt-in cross-application HMR

Farm hosts and remotes can enable `hot: true` on the Federation plugin in development. Enable the ordinary Farm HMR server in each app, set each remote's explicit HMR host/port, and allow the exact host-page origin through `server.allowedHosts`. For a remote on 4101 consumed by a host on 4100:

```js
server: {
  host: '127.0.0.1',
  port: 4101,
  hmr: { host: '127.0.0.1', port: 4101 },
  allowedHosts: ['http://127.0.0.1:4100'],
},
compilation: { css: { transformToScript: true } },
// Include hot: true in the existing federation({...}) options on both sides.
```

The host must explicitly accept each consumed remote boundary:

```js
import { bridge } from 'farm:module-federation';
const remote = await import('remote/Button');
const unsubscribe = bridge.acceptRemote('remote/Button', updated => {
  // Update the application's stable render boundary using updated exports.
  renderRemote(updated);
});
import.meta.hot?.dispose(unsubscribe);
```

Rust generates expose accept boundaries; the remote's Farm HMR client invalidates its own module cache and applies the update. The thin bridge updates a stable namespace used by the host's cached remote proxy and awaits registered consumer callbacks. Shared dependency negotiation remains with the official MF runtime. CSS DOM identities now include the runtime namespace, so identical `button.css` module IDs in different remotes do not overwrite each other's styles. Explicit `transformToScript: false` in a hot remote dev build is rejected.

The Mac browser E2E covers global and ESM entries: JS/CSS changes, simultaneous two-remote updates, unchanged React singleton identity, host/remote state retention, compile errors, execution rollback and compatible repairs. A document token and navigation counter prove that these updates do not reload the host. The React fixture uses an explicit stable component boundary with unchanged Hooks structure; this is not a general React Fast Refresh/signature implementation. Consumer callbacks must handle their own rendering and recovery; arbitrary external side effects are not automatically reversible.

`farm:mf:hot-status` browser events report `applied`, `compile-error`, `rolled-back` and `reload-required`. Incompatible export shapes or missing consumer acceptance do not silently count as success. A compatible source repair can recover after rollback; otherwise refresh explicitly. A lost remote connection uses Farm's existing reconnect/full-document-reload policy to avoid missing updates; that fallback resets state and is separately asserted, followed by another successful no-refresh update. Shared-library/configuration changes should use full reload.

HMR is currently Farm-to-Farm, with explicit accepted boundaries. Rspack/Webpack results below cover loading, not cross-bundler HMR. This path uses the existing Farm development HMR transport and CSS injection; strict production CSP checks do not establish CSP-compatible HMR. No browser or OS security policy was relaxed. `hot` is disabled in production regardless of the option.

## Global and ESM interoperability

For ESM, change the producer's `format` and the consumer's remote descriptor `format` to `'esm'`. Do not label a script entry as ESM or vice versa. ESM output also publishes the named global in this prototype. Global output checks container and Farm namespace conflicts before execution; ESM dependencies may execute before its container guard, so uniqueness is still required.

| Producer → consumer | Global/script | ESM |
| --- | --- | --- |
| Farm → Farm (production and dev) | Verified | Verified |
| Rspack → Farm / Farm → Rspack | Verified | Verified |
| Webpack → Farm / Farm → Webpack | Verified | Verified |

These are browser checks with real component rendering, shared React/JSX identity, CSS and lazy chunks. Versions are React/ReactDOM 18.3.1, MF runtime/enhanced 2.9.2, Rspack 2.3.0, Webpack 5.112.2, Playwright Chromium 1.55.0; Webpack CSS uses css-loader 7.1.5 and mini-css-extract-plugin 2.10.2. Production tests use CSP without `unsafe-eval` or `unsafe-inline`.

Rspack uses its MF enhanced adapter. Webpack tests use **Webpack's built-in ModuleFederationPlugin**: enhanced 2.9.2 references `webpack/lib/util/extractUrlAndGlobal`, which is absent in Webpack 5.112.2, so that enhanced/Webpack combination is not supported by this result. Script remotes use `globalName@URL`; module externals use a plain URL and a target supporting dynamic import (the tests use web + ES2020). See the local example's `interop-build.mjs` for the exact configurations. Mixed-format remotes in one application have not been separately verified.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Missing `index.farm` or incompatible core | Build the current-platform plugin and matching workspace core; do not substitute an unrelated binary. |
| Missing `index.html` in remote build | Use `input: { remoteEntry: 'farm-mf-native:container' }`. |
| Unknown remote alias/expose | Match alias/name/entry/globalName and `./Button` versus `remote/Button`. |
| Entry succeeds but CSS/chunks fail | Inspect remote publicPath, deployment subpath, CORS and resource status/MIME types. |
| Duplicate container global/runtime namespace | Choose unique names; the global container refuses to overwrite an existing runtime. |
| Shared version error or invalid hook call | Check version/requiredVersion, configure singleton on both sides and share the exact React/JSX requests. |
| Different share scope | Reuse the same scope object; creating a second scope is not a supported reinitialization. |
| Cannot synchronously require an async module | Use an explicit async import boundary; do not hide the diagnostic with a second unshared React copy. |

## Current limits

Global entry assembly requires `sourcemap: false`, accepts the generated eager side-effect import shape and leaves redundant eager resource files on disk. General global source maps and optimized entry assembly remain work in progress. SSR, manifests, type generation, automatic cross-framework HMR, multiple share scopes, version-first strategy, eager shares and wildcard shared subpath requests are not supported.

Ordinary synchronous CJS cycles and lazy remote A→B→A are tested. Static async CJS cycles and async dependencies required inside ordinary synchronous functions/class initializers produce compile diagnostics; arbitrary asynchronous cyclic evaluation is not implemented. The suite also retains non-MF CJS/ESM regression tests. See [runtime plugins](../api/runtime-plugin-api.md) and [dev server configuration](../config/dev-server.md) for the underlying Farm concepts.
