# Library Bundling

Farm supports building projects as a **library** — producing output intended for consumption by other packages rather than running directly in a browser or Node.js environment.

Set [`output.targetEnv`](/docs/config/compilation-options#output-targetenv) to `"library"` to enable library mode:

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

## Output Formats

In library mode you can produce multiple module formats in a single build by passing an array to [`output.format`](/docs/config/compilation-options#output-format):

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

Supported values: `"esm"`, `"cjs"`, `"umd"`, `"iife"`, `"system"`, `"amd"`.

## Bundle Types

Farm's compiler has three library bundle strategies, described in [`output.libraryBundleType`](/docs/config/compilation-options#output-librarybundletype):

:::note
The V2 candidate JavaScript API accepts `compilation.output.libraryBundleType` when `targetEnv` is `library`. Choose `single-bundle`, `multiple-bundle`, or `bundle-less`; omitting the field preserves the compiler's `single-bundle` default. Use matching candidate packages until the formal V2 release is published.
:::

### `single-bundle` (default)

All source modules are merged into **one output file** per format. This is the simplest option and is ideal for small libraries or when you want consumers to get a single file.

:::note
`single-bundle` supports only a single entry. If you configure multiple entries, Farm will throw an error.
:::

### `multiple-bundle`

Each entry produces its own output bundle. Internal modules that are shared between entries are extracted into separate shared chunks, similar to code-splitting for applications.

### `bundle-less`

Each source file is compiled independently and emitted as its own output file, **preserving the original directory structure**. This is the recommended approach for component libraries because it allows consumers to import individual modules and benefit from tree-shaking.

With `bundle-less`, the output retains the source structure. The following filenames are schematic: default output can include content hashes; use `output.filename` and `output.entryFilename` to control naming.

```
src/
  index.ts
  Button.tsx
  utils/
    format.ts
```

becomes:

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
For most UI component libraries, `bundle-less` is the preferred choice. It produces one file per source module, which enables consumers to import exactly what they need without importing the entire library.
:::

## Externals

Mark dependencies that should not be bundled into the library output using [`compilation.external`](/docs/config/compilation-options#externals):

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
    },
    external: ['react', 'react-dom'],
  },
});
```

For `iife` or `umd` formats, you can map the external module names to global variable names via [`output.externalGlobals`](/docs/config/compilation-options#outputexternalglobals):

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
    external: ['react', 'react-dom'],
  },
});
```

## Output Filename

Customize output filenames with [`output.entryFilename`](/docs/config/compilation-options#outputentryfilename) and [`output.filename`](/docs/config/compilation-options#outputfilename). Template tokens like `[entryName]`, `[resourceName]`, `[ext]`, and `[contentHash]` are supported:

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
