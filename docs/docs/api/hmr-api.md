# Hmr Api
:::note
The Farm HMR API is compatible with [Vite's HMR API](https://vitejs.dev/guide/api-hmr.html).
:::

Farm exports its HMR API via the special `import.meta.hot` object(compatible with Vite):
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

## Required Conditional Guard
HMR only works for development mode, make sure to guard HMR API usage with a conditional block:

```ts
if (import.meta.hot) {
  // HMR Code
}
```

## IntelliSense for TypeScript
The same as Vite, Farm provides type definitions for `import.meta.hot` in `farm/client.d.ts`. You can create an `env.d.ts` in the src directory so TypeScript picks up the type definitions:

```ts
/// <reference types="farm/client" />
```

## hot.accept()
For a self-accepted module, use `import.meta.hot.accept()`:

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
If you want to update the module status based on `exports of updated module`, you can use `import.meta.hot.accept(cb)`:

```ts
if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    const comp = mod[id]().render();
    div?.replaceWith(comp);
  });
}
```
Arguments of `cb` is the `exports of updated module`, you can do updates based on it.

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
A self-accepting module or a module that expects to be accepted by others can use hot.dispose to clean-up any persistent side effects created by its updated copy:

```ts
if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    div?.appendChild(mod.createChild());
  });

  // clean side effects
  import.meta.hot.dispose(() => {
    // remove all children of the div
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
Register a callback that will call when the module is no longer imported on the page. Compared to hot.dispose, this can be used if the source code cleans up side-effects by itself on updates and you only need to clean-up when it's removed from the page. Farm currently uses this for .css imports(the same as Vite).

```ts
if (import.meta.hot) {{
  import.meta.hot.accept();
  import.meta.hot.prune(() => {{
    style.remove();
  }});
}}
```

## hot.data

The import.meta.hot.data object is persisted across different instances of the same updated module. It can be used to pass on information from a previous version of the module to the next one.

```ts
import.meta.hot.data.value = 'value';
```

## hot.invalidate(message?: string)
A self-accepting module may realize during runtime that it can't handle a HMR update, and so the update needs to be forcefully propagated to importers. By calling import.meta.hot.invalidate(), the HMR server will invalidate the importers of the caller, as if the caller wasn't self-accepting. This will log a message both in the browser console and in the terminal. You can pass a message to give some context on why the invalidation happened.

Note that you should always call import.meta.hot.accept even if you plan to call invalidate immediately afterwards, or else the HMR client won't listen for future changes to the self-accepting module. To communicate your intent clearly, we recommend calling invalidate within the accept callback like so:

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
The same as Vite, see [Vite hot.on](https://vitejs.dev/guide/api-hmr.html#hot-on-event-cb)

## hot.off(event, cb)
Remove callback from the event listeners

## hot.send(event, data)
Send message from HMR client to dev server:

```ts
import.meta.hot.send('event-name', { data: '123' });
```

Receive message on dev server:

```ts
server.ws.on('event-name', (data) => {});
```