# Hmr Api

:::note
HMR API 兼容[Vite 的 HMR API](https://vitejs.dev/guide/api-hmr.html)。
:::

Farm 通过特殊的 `import.meta.hot` 对象导出 HMR API（与Vite兼容）：

```ts
export interface ViteHotContext {
  readonly data: any;

  accept(): void;
  accept(cb: (mod: ModuleNamespace | undefined) => void): void;
  accept(dep: string, cb: (mod: ModuleNamespace | undefined) => void): void;
  accept(
    deps: readonly string[],
    cb: (mods: Array<ModuleNamespace | undefined>) => void,
  ): void;

  dispose(cb: (data: any) => void): void;
  prune(cb: (data: any) => void): void;
  invalidate(message?: string): void;

  on<T extends string>(
    event: T,
    cb: (payload: InferCustomEventPayload<T>) => void,
  ): void;
  off<T extends string>(
    event: T,
    cb: (payload: InferCustomEventPayload<T>) => void,
  ): void;
  send<T extends string>(event: T, data?: InferCustomEventPayload<T>): void;
}
```

## HMR 前置判断

HMR 仅适用于开发模式，请确保使用条件块保护 HMR API 使用：

```ts
if (import.meta.hot) {
  // HMR Code
}
```

## Typescript 支持

和 Vite 一样，Farm 在 farm/client.d.ts 中提供了 import.meta.hot 的类型定义。 您可以在 src 目录中创建一个 `env.d.ts` ，以便 TypeScript 获取类型定义：

```ts
/// <reference types="farm/client" />
```

## hot.accept()

对于接收自身更新的模块，请使用 `import.meta.hot.accept()` ：

```ts
if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept();

  const div = document.getElementById(id);
  // update the page
  if (div) {
    const comp = SelfAcceptedEmpty().render();
    div.replaceWith(comp);
  }
}
```

## hot.accept(cb)

如果你想根据 `更新模块的导出` 来更新模块状态，可以使用 `import.meta.hot.accept(cb)` ：

```ts
if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept((mod) => {
    const div = document.getElementById(id);
    const comp = mod[id]().render();
    div?.replaceWith(comp);
  });
}
```

`cb` 的参数是 `更新模块的导出` ，您可以基于它进行更新。

## hot.accept(deps, cb)

模块还可以接受来自直接依赖项的更新，而无需重新加载自身。

接受单一依赖：

```ts
if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept("./accept-deps-data", (data) => {
    console.log(data);
    const div = document.getElementById(id);
    const renderData = data.compData(id);
    div!.innerText = renderData;
  });
}
```

接受多个依赖项：

```ts
if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept(["./accept-deps-data"], ([data]) => {
    console.log(data);
    const div = document.getElementById(id);
    const renderData = data.compData(id);
    div!.innerText = renderData;
  });
}
```

## hot.dispose(cb)

自我接受模块或期望被其他模块接受的模块可以使用 hot.dispose 来清理其更新副本所产生的任何持久副作用：

```ts
if (import.meta.hot) {
  // 接受 HMR 更新以避免页面刷新
  import.meta.hot.accept((mod) => {
    const div = document.getElementById(id);
    div?.appendChild(mod.createChild());
  });

  // 清理副作用
  import.meta.hot.dispose(() => {
    // 删除div的所有子元素
    const div = document.getElementById(id);

    if (div) {
      while (div.firstChild) {
        console.log("dispose", div.firstChild);
        div.removeChild(div.firstChild);
      }
    }
  });
}
```

## hot.prune(cb)

注册一个回调，当页面上不再导入模块时将调用该回调。 与 hot.dispose 相比，如果源代码在更新时自行清除副作用，并且您只需要在从页面中删除它时进行清除，则可以使用此方法。 Farm 目前使用它来导入 .css（与 Vite 相同）。

```ts
if (import.meta.hot) {
  {
    import.meta.hot.accept();
    import.meta.hot.prune(() => {
      {
        style.remove();
      }
    });
  }
}
```

## hot.data

import.meta.hot.data 对象在同一更新模块的不同实例中保留。 它可用于将信息从模块的前一版本传递到下一版本。

```ts
import.meta.hot.data.value = "value";
```

## hot.invalidate(message?: string)

自接受模块可能会在运行时意识到它无法处理 HMR 更新，因此需要将更新强制传播到祖先模块。 通过调用 import.meta.hot.invalidate()，HMR 服务器将使调用者的接收更新状态失效，并将此次更新通知以此到所有祖先模块，如果有任意祖先模块接收本次更新，HMR 成功，否则，将会刷新页面。 这将在浏览器控制台和终端中打印一条消息。 您可以传递一条消息来提供有关失效发生原因的一些背景信息。

请注意，即使您计划随后立即调用 invalidate，您也应该始终调用 import.meta.hot.accept，否则 HMR 客户端将不会监听自我接受模块的未来更改。 为了清楚地传达您的意图，我们建议在接受回调中调用 invalidate，如下所示：

```ts
if (import.meta.hot) {
  import.meta.hot.accept((mod) => {
    if (cannotHandleUpdate(mod)) {
      import.meta.hot.invalidate("parent module should accept this");
    }
  });
}
```

## hot.on(event, cb)

与Vite相同，参见 [Vite hot.on](https://vitejs.dev/guide/api-hmr.html#hot-on-event-cb)

## hot.off(event, cb)

从事件监听器中删除回调

## hot.send(event, data)

从 HMR 客户端向开发服务器发送消息：

```ts
import.meta.hot.send("event-name", { data: "123" });
```

在开发服务器上接收消息：

```ts
server.ws.on("event-name", (data) => {});
```
