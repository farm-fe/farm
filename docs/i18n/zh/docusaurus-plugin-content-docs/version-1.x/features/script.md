# TS/TSX

Farm 支持开箱即用地编译`Js/Jsx/Ts/Tsx`，并默认将`Jsx/Tsx`编译为 React。

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

Farm 使用 SWC 来编译脚本，Farm 为脚本编译设置了合理的默认配置。 另外，您可以使用`compilation.script`来配置如何编译脚本文件。 有关详细信息，请参阅 [compilation.script](/docs/config/farm-config#compilation-options)。

## 配置 Swc 解析器

您可以通过`compilation.script.parser`配置 SWC 解析器。 请参阅 https://swc.rs/docs/configuration/compilation#jscparser。

例如，如果你想启用装饰器，你可以设置`compilation.script.parser.esConfig.decorators`（如果模块是 TS，则设置 tsConfig.decorators）：

```ts title="farm.config.ts"
export default {
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
};
```

默认情况下，Farm 为`.jsx|.tsx`文件设置`jsx: true`。 其他字段默认为 SWC 的默认值。

## 配置目标执行环境

运行项目时使用`compilation.script.target`配置目标环境，Farm 将其默认设置为`ESNext`。

此选项可以与`compilation.presetEnv`一起使用，以针对旧浏览器优雅地降级您的项目。 例如，您可以将 target 设置为 `ES5` 并启用 `presetEnv`，那么您的项目将完全降级到 ES5。

```ts title="farm.config.ts"
export default {
  compilation: {
    script: {
      target: "ES5",
    },
    presetEnv: true,
  },
};
```

有关`presetEnv`的更多信息，请参阅 [Polyfill](/docs/features/polyfill)。

## 装饰器

装饰器默认不启用, 可以通过设置 `compilation.script.parser.tsConfig.decorators` 为 `true` 来启用装饰器。

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    script: {
      parser: {
        tsConfig: {
          // 启用装饰器
          decorators: true,
        },
      },
      // 配置装饰器
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

> Farm 提供了一个装饰器的示例，可以看 https://github.com/farm-fe/farm/tree/main/examples/decorators

> 默认情况下, Farm 不会转译 `node_modules` 下的装饰器, 参考 [compilation.script.decorators.excludes](/docs/config/farm-config#scriptdecorators).

## 使用 SWC 插件

SWC Plugins 可以直接在 Farm 中使用，例如我们在 Farm 中使用 swc-plugin-vue-jsx 来编译 vue jsx：

```ts title="farm.config.ts"
import jsPluginVue from "@farmfe/js-plugin-vue";

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
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
};
```

有关更多详细信息，请参阅[使用插件](/docs/using-plugins#using-swc-plugins)。

## Vite 风格的 `import.meta.glob`

Farm 完整支持 Vite 风格的 `import.meta.glob`, 参考 [glob import](https://vitejs.dev/guide/features.html#glob-import).

例如:

```ts
const modules = import.meta.glob("./dir/*.js");
```

将会被编译成以下结果

```ts
// code produced by Farm
const modules = {
  "./dir/foo.js": () => import("./dir/foo.js"),
  "./dir/bar.js": () => import("./dir/bar.js"),
};
```

使用 `{ eager: true }` 后:

```ts
const modules = import.meta.glob("./dir/*.js", { eager: true });
```

将会被编译成以下结果:

```ts
// code produced by Farm
import * as __glob__0_0 from "./dir/foo.js";
import * as __glob__0_1 from "./dir/bar.js";
const modules = {
  "./dir/foo.js": __glob__0_0,
  "./dir/bar.js": __glob__0_1,
};
```

支持数组形式:

```ts
const modules = import.meta.glob(["./dir/*.js", "./another/*.js"]);
```

支持通过 `!` 排除某些匹配:

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

- `import.meta.glob` 参数必须全部是字面量，不能使用表达式。
- `import.meta.glob` 在编译时处理和转换，在运行时不存在。
  :::
