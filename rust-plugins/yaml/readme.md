# @farmfe/plugin-yaml

Inspired By [@rollup/plugin-yaml](https://www.npmjs.com/package/@rollup/plugin-yaml)

🍣 A Farm plugin which Converts YAML files to ES6 modules.

## install

```bash
pnpm add -D @farmfe/plugin-yaml
```

## Usage

farm.config.ts

```typescript
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: [
    [
      '@farmfe/plugin-yaml',
      {
        documentMode: 'single' | 'multi', // default single
        include: Regex, // default None,
        exclude: Regex, // default None
      },
    ],
  ],
});
```

*notice:*

include or exclude is Regex not glob For example `**/01.yaml` is not illegal。What is right is like `".*\\/01.yaml"`
