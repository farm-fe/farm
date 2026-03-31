# Library 打包

Farm 支持将项目构建为 **库（library）** ——即生成供其他包消费的输出，而非直接在浏览器或 Node.js 环境中运行。

将 [`output.targetEnv`](/zh/docs/config/compilation-options#output-targetenv) 设置为 `"library"` 即可启用库模式：

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: 'esm',
    },
  },
});
```

## 输出格式

在库模式下，你可以通过向 [`output.format`](/zh/docs/config/compilation-options#outputformat) 传入数组，在一次构建中同时产出多种模块格式：

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
    },
  },
});
```

支持的格式：`"esm"`、`"cjs"`、`"umd"`、`"iife"`、`"system"`、`"amd"`。

## 打包类型

使用 [`output.libraryBundleType`](/zh/docs/config/compilation-options#output-librarybundletype) 控制模块如何分组到输出文件中。共有三种模式：

### `single-bundle`（默认）

将所有源模块合并为 **每种格式一个输出文件**。这是最简单的选项，适合小型库或希望消费者只拿到单一文件的场景。

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
      libraryBundleType: 'single-bundle',
    },
  },
});
```

:::note
`single-bundle` 仅支持单一入口。若配置了多个入口，Farm 会抛出错误。
:::

### `multiple-bundle`

每个入口产出独立的打包文件。入口之间共享的内部模块会被提取为独立的共享 chunk，类似于应用的代码分割。

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts',
      utils: './src/utils.ts',
    },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
      libraryBundleType: 'multiple-bundle',
    },
  },
});
```

### `bundle-less`

每个源文件独立编译并产出为自己的输出文件，**保留原始目录结构**。这是组件库的推荐方式，消费者可以按需导入单个模块，从而享受 tree-shaking 优化。

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
      libraryBundleType: 'bundle-less',
    },
  },
});
```

使用 `bundle-less` 时，输出结构与源码结构一一对应：

```
src/
  index.ts
  Button.tsx
  utils/
    format.ts
```

对应产出：

```
dist/
  index.js      (esm)
  Button.js
  utils/
    format.js
  index.cjs     (cjs)
  Button.cjs
  utils/
    format.cjs
```

:::tip
对于大多数 UI 组件库，`bundle-less` 是首选方案。它为每个源模块产出一个文件，让消费者可以精确地按需导入，而无需加载整个库。
:::

## 外部依赖（Externals）

使用 [`compilation.externals`](/zh/docs/config/compilation-options#externals) 将不应打入库的依赖标记为外部依赖：

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
    },
    externals: ['react', 'react-dom'],
  },
});
```

对于 `iife` 或 `umd` 格式，可通过 [`output.externalGlobals`](/zh/docs/config/compilation-options#outputexternalglobals) 将外部模块名映射为全局变量名：

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: 'umd',
      name: 'MyLibrary',
      externalGlobals: {
        react: 'React',
        'react-dom': 'ReactDOM',
      },
    },
    externals: ['react', 'react-dom'],
  },
});
```

## 输出文件名

通过 [`output.entryFilename`](/zh/docs/config/compilation-options#outputentryfilename) 和 [`output.filename`](/zh/docs/config/compilation-options#outputfilename) 自定义输出文件名。支持 `[entryName]`、`[ext]`、`[hash]`、`[contentHash]` 等模板占位符：

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
      entryFilename: '[entryName].[ext]',
      filename: '[resourceName].[ext]',
    },
  },
});
```
