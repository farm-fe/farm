# Plugins Options
Configure Farm's plug-ins, support Rust plug-ins or Js plug-ins, examples are as follows:

```ts
import type { UserConfig } from '@farmfe/core';
import less from '@farmfe/js-plugin-less';

function defineConfig(config: UserConfig) {
   return config;
}

export default defineConfig({
   plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass', less()],
});
```

### Rust Plugins
* **default**: `[]`

Rust plugins are configured via `plugin name` or `[plugin name, configuration option]`, as follows:

```ts
import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
   return config;
}

export default defineConfig({
   plugins: [['@farmfe/plugin-react', { /* options here */}], '@farmfe/plugin-sass'],
});
```

### Js Plugins
* **default**: `[]`

Js plugin is an object, for details, please refer to [Js plugin](/docs/plugins/js-plugin)
