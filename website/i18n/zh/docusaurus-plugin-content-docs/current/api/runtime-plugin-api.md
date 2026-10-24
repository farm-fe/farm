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

## 资源映射与事务恢复

`moduleSystem.gd()` 返回独立的资源映射快照；`sd(resources, moduleResources)` 恢复映射。它们不会撤销已执行的 JavaScript 或插件任意副作用。

`await moduleSystem.beginResourceTransaction()` 先等待此前的导入，再打开资源批次。批次提供异步 `validate()` 与同步 `finalize()`，也可使用便利方法 `commit()`。新增跟踪任务或映射变更会使此前的就绪状态失效。失败时，`rollback()` 等待未完成请求并恢复该批次的资源状态；模块工厂、缓存与插件恢复需要另外协调。同一时间只允许一个资源事务。

HMR 的 `import.meta.hot.registerUpdateParticipant(prepare)` 可注册包含 `validate`、可选 `ready`、同步 `commit`、`rollback`、`recover` 的参与者。可能失败的异步检查应放在验证阶段，不放在最终提交中。回滚不等于深复制模块导出，也不能撤销任意副作用。

## 钩子

:::caution
公开类型允许返回 Promise，但同步 `require` 路径不会等待运行时钩子。尤其 `readModuleCache` 与 `moduleNotFound` 使用 first/bail 分发，应返回同步结果；Promise 本身为 truthy，不代表异步模块解析协议。异步资源加载应使用 dynamic import 与 `loadResource`。
:::
那么调用Farm运行时插件钩子时有两种执行顺序：
* `serial`: 该钩子按照插件的顺序依次调用ono。 所有插件都会被串行调用。
* `first`: 一旦返回`truthy`值就跳过所有剩下的插件。

钩子执行顺序：
<a class="farm-docs-diagram farm-docs-diagram-light" href="/diagrams/runtime-hook-order-light.svg"><img src="/diagrams/runtime-hook-order-light.svg" alt="运行时钩子顺序与缓存分支" /></a>
<a class="farm-docs-diagram farm-docs-diagram-dark" href="/diagrams/runtime-hook-order-dark.svg"><img src="/diagrams/runtime-hook-order-dark.svg" alt="运行时钩子顺序与缓存分支" /></a>

[Mermaid 源文件](/diagrams/runtime-hook-order.mmd)
### name
- **`type`**: `string`

您的运行时插件的名称。

### bootstrap
- **`hook type`**: `serial`
- **`type`**: <code data-language="ts">(moduleSystem: ModuleSystem) =&gt; void | Promise&lt;void&gt;</code>

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
- **`type`**: <code data-language="ts">(module: Module) =&gt; void | Promise&lt;void&gt;</code>

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
- **`type`**: <code data-language="ts">(module: Module) =&gt; void | Promise&lt;void&gt;</code>

在调用模块初始化函数后调用。

:::note
`moduleCreated` 在模块执行后被调用，因此 `module.exports` 在此钩子中可用。
:::

### readModuleCache
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(module: Module) =&gt; boolean | Promise&lt;boolean&gt;</code>

读取模块缓存后调用，返回 true 以跳过缓存读取并重新执行模块。

### moduleNotFound
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(moduleId: string) =&gt; void | Promise&lt;void&gt;</code>

当模块未注册时调用。


### loadResource
- **`hook type`**: `first`
- **`type`**: <code data-language="ts">(resource: Resource, targetEnv: 'browser' | 'node') =&gt; Promise&lt;ResourceLoadResult&gt;</code>

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