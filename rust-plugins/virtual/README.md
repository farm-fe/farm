# @farmfe/plugin-virtual

Inspired By [@rollup/plugin-virtual](https://www.npmjs.com/package/@rollup/plugin-virtual)

🍣 A Rollup plugin which loads virtual modules from memory

## install

```bash
pnpm add @farmfe/plugin-virtual --save-dev # or yarn add @farmfe/plugin-virtual --save-dev
```

## Usage

farm.config.ts

```typescript
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: [
    [
      '@farmfe/plugin-virtual',
      {
        'virtual-module': 'export const a = 1',
        'src/01.js': 'export const module01 = "virtual-module"',
      },
    ],
  ],
});
```

index.js

```javascript
import { a } from 'virtual-module';
```

src/02.js

```javascript
import { module01 } from './01.js';
```
