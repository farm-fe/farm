
# Overview
To use a Rust plugin, configuring `plugins` in `farm.config.ts`.

```ts
import { defineFarmConfig } from 'farm';

defineFarmConfig({
  // ...
  plugins: [
    { /*..*/ }, // Js plugin, a object with hook defined
    '@farmfe/plugin-react', // rust plugin package name
  ]
})

```

Farm support both rust plugins and js plugins:

* [Writing Rust Plugin](/docs/plugins/writing-plugins/rust-plugin)
* [Writing Js Plugin](/docs/plugins/writing-plugins/js-plugin)
<!-- * [Writing Runtime Plugin](/docs/plugins/writing-plugins/runtime-plugin) -->