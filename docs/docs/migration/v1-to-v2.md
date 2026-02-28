---
sidebar_position: 1
---

# Migrating from Farm v1 to v2

Farm v2 introduces significant improvements and changes to the configuration system, plugin APIs, and overall architecture. This guide will help you migrate your Farm v1 project to v2.

## Overview of Changes

Farm v2 includes:
- **Improved Configuration Structure**: Cleaner config organization with top-level config fields
- **Enhanced Plugin System**: Improved Rust and JavaScript plugin APIs
- **Better HMR Handling**: Stricter origin validation for security
- **New Features**: UMD format, CSS transform to script, improved cache handling
- **Performance Improvements**: Faster builds and better module bundling

## Breaking Changes

### Configuration Changes

#### Config Structure Reorganization

In v1, custom configuration was placed under `config.custom`. In v2, these have been promoted to top-level configuration fields.

**v1:**
```javascript
// farm.config.ts (v1)
export default {
  config: {
    custom: {
      external: ['lodash'],
      // other custom settings
    }
  }
}
```

**v2:**
```javascript
// farm.config.ts (v2)
export default {
  // External dependencies are now at top level
  external: ['lodash'],
  // other settings
}
```

#### Configuration API Changes

The following configuration improvements were made:

- All `config.custom.*` fields have been promoted to top-level: `config.*`
- Type safety improvements for all configuration options
- Better default value handling

### HMR Origin Validation

**Change**: The HMR server now validates the `Origin` header from clients.

**v1 behavior**: HMR accepted connections from any origin

**v2 behavior**: HMR rejects connections with unrecognized Origin headers

**What to do if you get HMR errors:**
1. Ensure your client and server use matching hostnames/domains
2. Update your HMR configuration if needed:

```javascript
// farm.config.ts
export default {
  devServer: {
    hmr: {
      host: 'localhost',
      port: 9801,
    }
  }
}
```

### Plugin API Changes

#### Rust Plugin API

The Rust plugin trait has been enhanced with new hooks and improved semantics. See the [Rust Plugin API documentation](../api/rust-plugin-api) for detailed information.

Key changes:
- New hooks for module processing: `process_module_graph`, `optimize_module_graph`
- Improved hook context with more metadata
- Better error handling in hooks

#### JavaScript Plugin API

The JavaScript plugin interface has improved type safety and added new hooks. See the [JavaScript Plugin API documentation](../api/js-plugin-api) for detailed information.

Key changes:
- Type-safe plugin hooks with better TypeScript support
- New hooks for resource processing: `processRenderedResourcePot`, `augmentResourcePotHash`
- Consistent hook naming across frameworks

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

1. **Move `config.custom` settings to top level**
   ```javascript
   // Before (v1)
   export default {
     config: {
       custom: {
         external: ['lodash']
       }
     }
   }

   // After (v2)
   export default {
     external: ['lodash']
   }
   ```

2. **Test configuration validity**
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
