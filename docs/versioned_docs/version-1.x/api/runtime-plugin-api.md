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

## Hooks
Then are 2 kind of execution orders when calling Farm runtime plugin hooks:
* `serial`: The hook is called ono by one by the order of plugins. All plugins would be called serially.
* `first`:  Skip all left plugins once `truthy` value is returned.

Hook execution order:
```text
          for each module                     true                         true                     return false
bootstrap  ----------->   module registered? ------> module initialized?  ----> readModuleCache -------------------------> done
           |                   |              false            |false              | return true                            | 
           |                   |                               |--------------> moduleCreated   ------> moduleInitialized --|
           |                   |-------------------> moduleNotFound
           |
           | dynamic import
           | ---------------> loadResource                         
``` 

### name
- **`type`**: `string`

The name of your runtime plugin.

### bootstrap
- **`hook type`**: `serial`
- **`type`**: `(moduleSystem: ModuleSystem) => void | Promise<void>`

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
- **`type`**: `(module: Module) => void | Promise<void>`

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
- **`type`**: `(module: Module) => void | Promise<void>`

Invoked after module initialization functions are called.

:::note
`moduleCreated` is called **AFTER** the module is executed, so `module.exports` is available is this hook.
:::

### readModuleCache
- **`hook type`**: `serial`
- **`type`**: `(module: Module) => boolean | Promise<boolean>`

Invoked after module caches are read, return true to skip cache reading and re-executed the module.

### moduleNotFound
- **`hook type`**: `serial`
- **`type`**: `(module: Module) => void | Promise<void>`

Called when module is not registered.


### loadResource
- **`hook type`**: `first`
- **`type`**: `(resource: Resource, targetEnv: 'browser' | 'node') => Promise<ResourceLoadResult>`

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