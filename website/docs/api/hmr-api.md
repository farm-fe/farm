# HMR API
:::note
The Farm HMR API is compatible with [Vite's HMR API](https://vitejs.dev/guide/api-hmr.html).
:::

Farm exposes HMR through `import.meta.hot` (Vite-compatible surface):
```ts
export interface ViteHotContext {
  readonly data: any;

  accept(): void;
  accept(cb: (mod: ModuleNamespace | undefined) => void): void;
  accept(dep: string, cb: (mod: ModuleNamespace | undefined) => void): void;
  accept(
    deps: readonly string[],
    cb: (mods: Array<ModuleNamespace | undefined>) => void
  ): void;

  acceptExports(
    exportNames: string | readonly string[],
    cb?: (mod: ModuleNamespace | undefined) => void
  ): void;

  dispose(cb: (data: any) => void): void;
  prune(cb: (data: any) => void): void;
  invalidate(message?: string): void;

  on<T extends string>(
    event: T,
    cb: (payload: InferCustomEventPayload<T>) => void
  ): void;
  off<T extends string>(
    event: T,
    cb: (payload: InferCustomEventPayload<T>) => void
  ): void;
  send<T extends string>(event: T, data?: InferCustomEventPayload<T>): void;
}
```

## Compatibility Notes

- `acceptExports` exists in the type surface in v2, but Farm currently treats it as not supported (runtime logs a debug message).
- `on`, `off`, and `send` are available for custom HMR events.

## Required Conditional Guard
HMR only works in development mode. Always guard HMR usage:

```ts
if (import.meta.hot) {
  // HMR Code
}
```

## IntelliSense for TypeScript
As with Vite, Farm provides `import.meta.hot` types from `@farmfe/core/client`. Add an `env.d.ts` file so TypeScript picks them up:

```ts
/// <reference types="@farmfe/core/client" />
```

## hot.accept()
For a self-accepted module, use `import.meta.hot.accept()`:

```ts
if (import.meta.hot) {
  // Self-accept without full page reload
  import.meta.hot.accept();

  const div = document.getElementById(id);
  // Update the page
  if (div) {
    const comp = SelfAcceptedEmpty().render();
    div.replaceWith(comp);
  }
}
```

## hot.accept(cb)
If you want to update state based on the updated module exports, use `import.meta.hot.accept(cb)`:

```ts
if (import.meta.hot) {
  // Self-accept without full page reload
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    const comp = mod[id]().render();
    div?.replaceWith(comp);
  });
}
```
`cb` receives the updated module exports.

## hot.accept(deps, cb)
A module can also accept updates from direct dependencies without reloading itself.

Accept single dependency:
```ts
if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept('./accept-deps-data', (data) => {
    console.log(data);
    const div = document.getElementById(id);
    const renderData = data.compData(id);
    div!.innerText = renderData;
  });
}
```

Accept multiple dependencies:
```ts
if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept(['./accept-deps-data'], ([data]) => {
    console.log(data);
    const div = document.getElementById(id);
    const renderData = data.compData(id);
    div!.innerText = renderData;
  });
}
```

## hot.dispose(cb)
A self-accepting module (or a module accepted by others) can use `hot.dispose` to clean up persistent side effects from the previous instance:

```ts
if (import.meta.hot) {
  // Self-accept without full page reload
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    div?.appendChild(mod.createChild());
  });

  // Clean side effects
  import.meta.hot.dispose(() => {
    // Remove all children of the div
    const div = document.getElementById(id);
    
    if (div) {
      while (div.firstChild) {
        console.log('dispose', div.firstChild);
        div.removeChild(div.firstChild);
      }
    }
  });
}
```

## hot.prune(cb)
Register a callback that runs when the module is no longer imported on the page. Compared with `hot.dispose`, this is useful when update-time cleanup is already handled in code and you only need cleanup on removal. Farm currently uses this for CSS imports (same as Vite).

```ts
if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.prune(() => {
    style.remove();
  });
}
```

## hot.data

`import.meta.hot.data` persists across different instances of the same updated module. Use it to pass information from the previous instance to the next one.

```ts
import.meta.hot.data.value = 'value';
```

## hot.invalidate(message?: string)
A self-accepting module may detect at runtime that it cannot safely handle the update. Calling `import.meta.hot.invalidate()` forces propagation to importers as if the module were not self-accepting. You can pass a message for context.

You should still call `import.meta.hot.accept(...)`; otherwise the client will not continue listening for updates to the module.

```ts
if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept((mod) => {
    if (cannotHandleUpdate(mod)) {
      import.meta.hot.invalidate('parent module should accept this');
    }
  });
}
```

## hot.on(event, cb)
Same as Vite. See [Vite hot.on](https://vitejs.dev/guide/api-hmr.html#hot-on-event-cb).

## hot.off(event, cb)
Remove a callback from event listeners.

## hot.send(event, data)
Send a custom message from HMR client to dev server:

```ts
import.meta.hot.send('event-name', { data: '123' });
```

Receive message on dev server:

```ts
server.ws.on('event-name', (data) => {});
```