# Runtime Plugin API
插件钩子定义：

```ts
export interface FarmRuntimePlugin {
  // 插件名称
  name: string;
  // 模块系统启动时调用
  bootstrap?: (moduleSystem: ModuleSystem) => void | Promise<void>;
  // 新模块创建时调用
  moduleCreated?: (module: Module) => void | Promise<void>;
  // 新模块执行后调用
  moduleInitialized?: (module: Module) => void | Promise<void>;
  // 读取缓存时调用
  readModuleCache?: (module: Module) => boolean | Promise<boolean>;
  // 模块未注册时调用
  moduleNotFound?: (moduleId: string) => void | Promise<void>;
  // 加载资源时调用，在此钩子中自定义您的资源加载。
  // return { success: true } 表示该资源已成功加载。
  // return { success: false, retryWithDefaultResourceLoader: true } 表示此资源尚未成功加载，应使用默认资源加载器重试。
  loadResource?: (
    resource: Resource,
    targetEnv: 'browser' | 'node'
  ) => Promise<ResourceLoadResult>;
}
```

## 编写运行时插件
请参阅[编写运行时插件](/docs/plugins/writing-plugins/runtime-plugin)

## 钩子
那么调用Farm运行时插件钩子时有两种执行顺序：
* `serial`: 该钩子按照插件的顺序依次调用ono。 所有插件都会被串行调用。
* `first`: 一旦返回`truthy`值就跳过所有剩下的插件。

钩子执行顺序：
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

您的运行时插件的名称。

### bootstrap
- **`hook type`**: `serial`
- **`type`**: `(moduleSystem: ModuleSystem) => void | Promise<void>`

当模块系统引导时调用一次。 在此挂钩中设置您的插件。 例子：

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

创建新模块实例后调用。 您可以读取或更新新创建的模块的属性。

```ts
export default <Plugin>{
  name: 'farm-runtime-hmr-client-plugin',
  moduleCreated(module) {
    // 为每个模块创建一个 hot 上下文
    module.meta.hot = createHotContext(module.id, hmrClient);
  }
};
```

:::note
`moduleCreated` 在模块执行之前被调用，因此 `module.exports` 始终为空，如果要访问 `module.exports`，请使用 `moduleInitialized`。
:::

### moduleInitialized
- **`hook type`**: `serial`
- **`type`**: `(module: Module) => void | Promise<void>`

在调用模块初始化函数后调用。

:::note
`moduleCreated` 在模块执行后被调用，因此 `module.exports` 在此钩子中可用。
:::

### readModuleCache
- **`hook type`**: `serial`
- **`type`**: `(module: Module) => boolean | Promise<boolean>`

读取模块缓存后调用，返回 true 以跳过缓存读取并重新执行模块。

### moduleNotFound
- **`hook type`**: `serial`
- **`type`**: `(module: Module) => void | Promise<void>`

当模块未注册时调用。


### loadResource
- **`hook type`**: `first`
- **`type`**: `(resource: Resource, targetEnv: 'browser' | 'node') => Promise<ResourceLoadResult>`

加载资源时调用，在此钩子中自定义您的资源加载。
* 返回 `{ success: true }` 表示该资源已成功加载。
* return `{ success: false, retryWithDefaultResourceLoader: true }` 表示此资源尚未成功加载，应使用默认资源加载器重试。

```ts
import { Plugin } from '@farmfe/runtime';

export default <Plugin>{
  name: 'runtime-plugin-example',
  loadResource: (resource, targetEnv) => {
    // 覆盖默认资源加载
    // 从不同位置加载资源
    return import('./replaced.js').then(() => {
      return {
        success: true
      };
    });
  }
};

```