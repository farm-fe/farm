# TS/TSX
Farm support compiling `Js/Jsx/Ts/Tsx` out of box, and compile `Jsx/Tsx` to React by default.

```tsx title="./button.tsx"
import Button from "./Button";

function ButtonGroup(props: ButtonProps) {
  return (
    <div>
      {props.buttons.map((b) => (
        <Button>{b}</Button>
      ))}
    </div>
  );
}
```

Farm using SWC to compile scripts, and Farm has set reasonable default configurations for script compilation. Also, you can use `compilation.script` to configure how to compile your script file. see [compilation.script](/docs/config/compilation-options#script) for details.

## Configuring Swc Parser

You can configuring the SWC Parser through `compilation.script.parser`. Refer to https://swc.rs/docs/configuration/compilation#jscparser.

For example, if you want to enable decorator, you can set `compilation.script.parser.esConfig.decorators`(or `tsConfig.decorators` if the module is TS):

```ts title="farm.config.ts"
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    script: {
      // for .js/.jsx files
      esConfig: {
        decorators: true,
      },
      // for .ts/.tsx files
      tsConfig: {
        decorators: true,
      },
    },
  },
});
```

By default Farm set `jsx: true` for `.jsx|.tsx` files. Other field are default to SWC's defaults.

## Configuring Target

Using `compilation.script.target` to configure your target env when running your project, Farm set it based on [`output.targetEnv`](/docs/config/compilation-options#output-targetenv).
:::note
Farm set `compilation.script.target` automatically based on [`output.targetEnv`](/docs/config/compilation-options#output-targetenv). Normally you should not set `target` manually, use [`output.targetEnv`](/docs/config/compilation-options#output-targetenv) would be enough.
:::

This option can be used along with `compilation.presetEnv` to gracefully downgrade your project for old browsers. For example, you can set target to `ES5` and enable `presetEnv`, then your project will be fully downgrade to ES5.

```ts title="farm.config.ts"
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    script: {
      target: "ES5",
    },
    presetEnv: true,
  },
});
```

Refer to [Syntax Downgrade and Polyfill](/docs/advanced/polyfill) for more about `presetEnv` and `target`.


## Decorators

Decorators is disabled by default, you can set `compilation.script.parser.tsConfig.decorators` to `true` to enable decorators.

```ts
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    script: {
      parser: {
        tsConfig: {
          // support decorators
          decorators: true,
        },
      },
      // configuring decorators
      decorators: {
        legacyDecorator: true,
        decoratorMetadata: false,
        decoratorVersion: '2021-12',
        includes: ["src/broken.ts"],
        excludes: ['node_modules/'],
      }
    },
  },
});
```

> Farm provide a example for supporting decorators, see https://github.com/farm-fe/farm/tree/main/examples/decorators
> By default, Farm won't transform decorators for modules under `node_modules`, refer to [compilation.script.decorators.excludes](/docs/config/compilation-options#scriptdecorators).


## Using SWC Plugins

SWC Plugins can be used directly in Farm, for example, we use `swc-plugin-vue-jsx` to compiling vue jsx in Farm:

```ts title="farm.config.ts"
import { defineConfig } from 'farm';
import jsPluginVue from "@farmfe/js-plugin-vue";

export default defineConfig({
  compilation: {
    script: {
      plugins: [
        {
          name: "swc-plugin-vue-jsx",
          options: {
            transformOn: true,
            optimize: true,
          },
          filters: {
            // resolvedPaths: [".+"]
            moduleTypes: ["tsx", "jsx"],
          },
        },
      ],
    },
  },
  plugins: [jsPluginVue()],
});
```

Refer to [Using Plugins](/docs/using-plugins#using-swc-plugins) for more details.

## Vite-style `import.meta.glob`

Farm fully support Vite-style `import.meta.glob`, see [glob import](https://vitejs.dev/guide/features.html#glob-import).

for example:

```ts
const modules = import.meta.glob("./dir/*.js");
```

The above will be transformed into the following:

```ts
// code produced by Farm
const modules = {
  "./dir/foo.js": () => import("./dir/foo.js"),
  "./dir/bar.js": () => import("./dir/bar.js"),
};
```

Using `{ eager: true }`:

```ts
const modules = import.meta.glob("./dir/*.js", { eager: true });
```

The above will be transformed into the following:

```ts
// code produced by Farm
import * as __glob__0_0 from "./dir/foo.js";
import * as __glob__0_1 from "./dir/bar.js";
const modules = {
  "./dir/foo.js": __glob__0_0,
  "./dir/bar.js": __glob__0_1,
};
```

multiple patterns are supported:

```ts
const modules = import.meta.glob(["./dir/*.js", "./another/*.js"]);
```

negative patterns are also supported:

```ts
const modules = import.meta.glob(["./dir/*.js", "!**/bar.js"]);
```

```ts
// code produced by Farm
const modules = {
  "./dir/foo.js": () => import("./dir/foo.js"),
};
```

:::note

- You should also be aware that all the arguments in the import.meta.glob must be passed as literals. You can NOT use variables or expressions in them.
- `import.meta.glob` transformed by Farm in compile time, it does not exist in runtime.
  :::
