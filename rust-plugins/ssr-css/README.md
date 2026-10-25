# @farmfe/plugin-ssr-css

Native Rust CSS resource handling for Node-target Farm output. It removes CSS entries from the entry's initial runtime resources so the Node renderer does not evaluate stylesheets, while retaining emitted CSS artifacts and CSS Modules exports.

```ts
import ssrCss from '@farmfe/plugin-ssr-css';

const plugins = [ssrCss()];
```

The helper takes no options and returns a Farm native plugin descriptor. Browser-target entry resources are unchanged. This plugin does not collect HTML stylesheet links or implement a complete SSR server; the Farm Docs workflow handles its own resource manifest and static rendering.

Use the plugin with a matching Core/native distribution. Farm Docs package/platform and isolated consumer validation remains in progress; the presence of a platform manifest is not an ABI check. See LICENSE for Farm code.
