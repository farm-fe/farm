# Runtime Plugin API
Plugin hook definition:

```ts
export interface FarmRuntimePlugin {
  // plugin name
  name: string;
  // invoked when the module system is bootstrapped
  bootstrap?: (moduleSystem: ModuleSystem) => void | Promise<void>;
  // invoked after new module instances are created
  moduleCreated?: (module: Module) => void | Promise<void>;
  // invoked after module initialization functions are called
  moduleInitialized?: (module: Module) => void | Promise<void>;
  // invoked after module caches are read, return true to skip cache reading
  readModuleCache?: (module: Module) => boolean | Promise<boolean>;
  // called when module is not found
  moduleNotFound?: (moduleId: string) => void | Promise<void>;
  // called when loading resources, custom your resource loading in this hook.
  // return { success: true } to indicate that this resources have been loaded successfully.
  // return { success: false, retryWithDefaultResourceLoader: true } to indicate that this resources have not been loaded successfully and should be retried with the default resource loader.
  loadResource?: (
    resource: Resource,
    targetEnv: 'browser' | 'node'
  ) => Promise<ResourceLoadResult>;
}
```

## Writing Runtime Plugin
See [Writing Runtime Plugin](/docs/plugins/writing-plugins/runtime-plugin)

## Resource mapping and transaction recovery

`moduleSystem.gd()` returns a detached mapping snapshot; `sd(resources, moduleResources)` restores that mapping. These operations do not undo executed JavaScript or arbitrary plugin side effects.

`await moduleSystem.beginResourceTransaction()` drains prior imports before opening a resource batch. The batch supports asynchronous `validate()` followed by synchronous `finalize()`, or the convenience `commit()`. New tracked work or mapping changes invalidate readiness. On failure, `rollback()` drains outstanding requests and restores the batch's resource state; coordinate module factories, caches and plugin recovery separately. Only one resource transaction may be open at a time.

The HMR API `import.meta.hot.registerUpdateParticipant(prepare)` can coordinate a participant with `validate`, optional `ready`, synchronous `commit`, `rollback` and `recover` callbacks. All fallible asynchronous checks belong in validation, not finalization. Rollback is not a deep copy of module exports or a reversal of arbitrary side effects.

## Hooks

:::caution
The public types permit Promise returns, but the synchronous `require` path does not await runtime hooks. In particular, `readModuleCache` and `moduleNotFound` are first/bail hooks: use synchronous results there. A Promise is itself truthy and is not an asynchronous module-resolution protocol. Use dynamic import and `loadResource` for asynchronous resource loading.
:::
Then are 2 kind of execution orders when calling Farm runtime plugin hooks:
* `serial`: The hook is called one by one by the order of plugins. All plugins would be called serially.
* `first`:  Skip all left plugins once `truthy` value is returned.

Hook execution order:
<a class="farm-docs-diagram farm-docs-diagram-light" href="/diagrams/runtime-hook-order-light.svg"><img src="/diagrams/runtime-hook-order-light.svg" alt="Runtime hook order and cache branches" /></a>
<a class="farm-docs-diagram farm-docs-diagram-dark" href="/diagrams/runtime-hook-order-dark.svg"><img src="/diagrams/runtime-hook-order-dark.svg" alt="Runtime hook order and cache branches" /></a>

[Mermaid source](/diagrams/runtime-hook-order.mmd)
### name
- **`type`**: `string`

The name of your runtime plugin.

### bootstrap
- **`hook type`**: `serial`
- **`type`**: <code data-language="ts">(moduleSystem: ModuleSystem) =&gt; void | Promise&lt;void&gt;</code>

Invoked once when the module system is bootstrapped. Setup your plugin in this hook. Example:

```ts
export default <Plugin>{
  name: 'farm-runtime-hmr-client-plugin',
  // define hooks
  bootstrap(moduleSystem) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
};
```

### moduleCreated
- **`hook type`**: `serial`
- **`type`**: <code data-language="ts">(module: Module) =&gt; void | Promise&lt;void&gt;</code>

Invoked after new module instances are created. You can read or update property of the new created module.

```ts
export default <Plugin>{
  name: 'farm-runtime-hmr-client-plugin',
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id, hmrClient);
  }
};
```

:::note
`moduleCreated` is called **BEFORE** the module is executed, so `module.exports` is always empty, use `moduleInitialized` instead if you want to access `module.exports`.
:::

### moduleInitialized
- **`hook type`**: `serial`
- **`type`**: <code data-language="ts">(module: Module) =&gt; void | Promise&lt;void&gt;</code>

Invoked after module initialization functions are called.

:::note
`moduleCreated` is called **AFTER** the module is executed, so `module.exports` is available is this hook.
:::

### readModuleCache
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(module: Module) =&gt; boolean | Promise&lt;boolean&gt;</code>

Invoked after module caches are read, return true to skip cache reading and re-executed the module.

### moduleNotFound
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(moduleId: string) =&gt; void | Promise&lt;void&gt;</code>

Called when module is not registered.


### loadResource
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(resource: Resource, targetEnv: 'browser' | 'node') =&gt; Promise&lt;ResourceLoadResult&gt;</code>

called when loading resources, custom your resource loading in this hook.
* return `{ success: true }` to indicate that this resources have been loaded successfully.
* return `{ success: false, retryWithDefaultResourceLoader: true }` to indicate that this resources have not been loaded successfully and should be retried with the default resource loader.

```ts
import { Plugin } from '@farmfe/runtime';

export default <Plugin>{
  name: 'runtime-plugin-example',
  loadResource: (resource, targetEnv) => {
    // override default resource loading
    // load the resource from different location
    return import('./replaced.js').then(() => {
      return {
        success: true
      };
    });
  }
};

```