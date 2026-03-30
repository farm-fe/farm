---
sidebar_position: 1
---

# Migrating from Farm v1 to v2

Farm v2 introduces significant improvements and changes to the configuration system, plugin APIs, and overall architecture. This guide will help you migrate your Farm v1 project to v2.

## Overview of Changes

Farm v2 includes:
- **New Output Formats**: Added `umd`, `iife`, `system`, `amd` formats (and array of formats)
- **Enhanced Plugin System**: New Rust and JavaScript plugin hooks
- **Server Middleware Rewrite**: Replaced Koa with connect-style middleware
- **Better HMR Handling**: Stricter origin validation, new sub-options (`clientPort`, `path`, `timeout`, `overlay`, `protocol`)
- **New Features**: CSS `transformToScript`, CSS modules `localsConversion`, `minify.include`/`exclude`, `persistentCache.envs`, `persistentCache.globalBuiltinCacheKeyStrategy`
- **Performance Improvements**: Faster builds and better module bundling

## Breaking Changes

### Configuration Changes

#### `server` replaces `devServer`

In v2, the dev server configuration key has been renamed from `devServer` to `server`:

**v1:**
```ts
// farm.config.ts (v1)
export default {
  devServer: {
    port: 9000,
    hmr: true,
  }
}
```

**v2:**
```ts
// farm.config.ts (v2)
export default {
  server: {
    port: 9000,
    hmr: true,
  }
}
```

#### `server.spa` replaced by `server.appType`

The `spa` boolean option has been replaced with `appType`:

```ts
// v1: spa: true/false
// v2: appType: 'spa' | 'mpa' | 'custom'
export default {
  server: {
    appType: 'spa', // default
  }
}
```

#### New `server` options in v2

- `origin` — Set the origin of the dev server URLs
- `allowedHosts` — Restrict which hosts can access the server
- `middlewareMode` — Run the server in middleware mode (no HTTP server created)
- `preview` — Dedicated preview server configuration

### Server Middleware API Change

**Breaking**: Farm v2 uses [connect](https://github.com/senchalabs/connect) middleware instead of Koa.

**v1 (Koa):**
```ts
import { Middleware } from 'koa';

function myMiddleware(server): Middleware {
  return async (ctx, next) => {
    ctx.set('X-Custom', 'value');
    await next();
  };
}
```

**v2 (connect):**
```ts
function myMiddleware(server) {
  return (req, res, next) => {
    res.setHeader('X-Custom', 'value');
    next();
  };
}
```

Replace any `koa-*` middleware packages with their `connect`/`express`-compatible equivalents (e.g. `koa-compress` → `compression`).

### HMR Origin Validation

**Change**: The HMR server now validates the `Origin` header from clients.

**v1 behavior**: HMR accepted connections from any origin

**v2 behavior**: HMR rejects connections with unrecognized Origin headers

**What to do if you get HMR errors:**
1. Ensure your client and server use matching hostnames/domains
2. Update your HMR configuration if needed:

```ts
// farm.config.ts
export default {
  server: {
    hmr: {
      host: 'localhost',
      port: 9801,
    }
  }
}
```

### HMR New Sub-Options

v2 adds new HMR configuration options:
- `clientPort` — Port for the HMR client (useful behind proxies)
- `path` — Custom HMR endpoint path (default: `/__hmr`)
- `timeout` — Connection timeout in milliseconds
- `overlay` — Show/hide error overlay in browser (default: `true`)
- `protocol` — WebSocket protocol (`ws` or `wss`)

### Plugin API Changes

#### Rust Plugin API

The Rust plugin trait has been enhanced with new hooks. See the [Rust Plugin API documentation](../api/rust-plugin-api) for detailed information.

New hooks in v2:
- `freeze_module` — Called after a module is frozen
- `module_graph_build_end` — Called when the module graph build is complete
- `freeze_module_graph_meta` — Freeze module graph metadata
- `process_resource_pots` — Process resource pots after bundling
- `process_rendered_resource_pot` — Process a resource pot after rendering
- `augment_resource_pot_hash` — Augment resource pot hash
- `process_generated_resources` — Process resources after generation
- `handle_entry_resource` — Handle entry resource processing
- `module_graph_updated` — Called after module graph is updated during HMR
- `update_finished` — Called when HMR update is finished
- `handle_persistent_cached_module` — Handle loading a module from persistent cache

#### JavaScript Plugin API

The JavaScript plugin interface has added new hooks. See the [JavaScript Plugin API documentation](../api/js-plugin-api) for detailed information.

New hooks in v2:
- `freezeModule` — Called after a module is frozen
- `processRenderedResourcePot` — Process a resource pot after rendering
- `augmentResourcePotHash` — Augment resource pot hash
- `finalizeResources` — Finalize all resources before output
- `updateFinished` — Called when HMR update is finished

## Step-by-Step Migration Guide

### 1. Update Farm Version

```bash
# Update to Farm v2
npm install @farmfe/core@latest @farmfe/cli@latest
# or
pnpm install @farmfe/core@latest @farmfe/cli@latest
```

### 2. Update Configuration File

Review your `farm.config.ts` or `farm.config.js`:

1. **Rename `devServer` to `server`**
   ```ts
   // Before (v1)
   export default { devServer: { port: 9000 } }
   // After (v2)
   export default { server: { port: 9000 } }
   ```

2. **Replace `spa` with `appType`**
   ```ts
   // Before (v1)
   server: { spa: true }
   // After (v2)
   server: { appType: 'spa' }
   ```

3. **Update middleware to connect-style** (see [Server Middleware API Change](#server-middleware-api-change))

4. **Test configuration validity**
   ```bash
   farm build
   ```

### 3. Update Plugins

If you're using custom plugins:

1. **Rust plugins**: Review the [Rust Plugin API documentation](../api/rust-plugin-api) for any hook signature changes
2. **JavaScript plugins**: Review the [JavaScript Plugin API documentation](../api/js-plugin-api) for any hook signature changes

For official plugins, ensure they're v2-compatible:

```bash
npm install @farmfe/plugin-{feature}@latest
```

### 4. Verify Development Server

Test that HMR is working correctly:

```bash
farm dev
```

If you encounter HMR connection errors:
- Check browser console for specific error messages
- Verify that `devServer.hmr` settings match your deployment environment
- Ensure no origin-related proxies are interfering

### 5. Test Build Output

```bash
farm build
```

Verify that:
- All assets are generated correctly
- Code splitting is working as expected
- Source maps are generated (if enabled)

## Configuration Migration Reference

| v1 Setting | v2 Location | Notes |
|-----------|-----------|-------|
| `config.custom.external` | `config.external` | Moved to top level |
| `config.custom.*` | `config.*` corresponding field | All custom fields promoted |
| `devServer.hmr` | `devServer.hmr` | Same, but now validates Origin header |

## Troubleshooting Common Issues

### "HMR connection rejected with origin check failed"

**Cause**: The server is rejecting your HMR client connection due to origin mismatch.

**Solution**:
1. Check that your client and server are using the same host/domain
2. Configure HMR explicitly if needed:
   ```javascript
   {
     devServer: {
       hmr: {
         host: 'your-domain.com',
         port: 9801
       }
     }
   }
   ```

### Configuration errors for moved settings

**Cause**: You're still using the old `config.custom.*` structure.

**Solution**: Move settings to top-level `config`:
```javascript
// Old (v1)
export default {
  config: {
    custom: {
      myOption: value
    }
  }
}

// New (v2)
export default {
  myOption: value
}
```

### Plugin hooks not being called

**Cause**: Plugin hook signatures or names have changed.

**Solution**: Check the updated [Plugin API documentation](../api/) for the latest hook specifications.

## Getting Help

- Review the [Configuration documentation](../config/configuring-farm)
- Check the [Plugin documentation](../plugins/overview)
- Visit the [Farm GitHub discussions](https://github.com/farm-fe/farm/discussions)
- Join the [Discord community](https://discord.com/invite/mDErq9aFnF)

## See Also

- [What's New in Farm v2](../features/v2-features)
- [Configuration Reference](../config/configuring-farm)
- [Rust Plugin API](../api/rust-plugin-api)
- [JavaScript Plugin API](../api/js-plugin-api)
