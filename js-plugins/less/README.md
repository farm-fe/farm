<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <span>English</span> |
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/less/README-zh-CN.md">简体中文</a>  
</div>

---

# Less Plugin for Farm

Support compiling Less in Farm.

## Getting Started

To begin, you'll need to install `less` and `@farmfe/js-plugin-less`:

```console
npm install less @farmfe/js-plugin-less --save-dev
```

or

```console
yarn add -D less @farmfe/js-plugin-less
```

or

```console
pnpm add -D less @farmfe/js-plugin-less
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';
import Less from '@farmfe/js-plugin-less'; //  import the plugin

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
  plugins: [
    // use the less plugin.
    Less({
      // custom options here
    }),
  ],
});
```

## Options

- **[`lessOptions`](#lessoptions)**
- **[`additionalData`](#additionalData)**
- **[`sourceMap`](#sourcemap)**
- **[`implementation`](#implementation)**

### lessOptions

Type: 
```ts
type lessOptions = import('less').options | ((loaderContext: LoaderContext) => import('less').options})
```

Default: `{ relativeUrls: true }`

Here you can pass any Less specific options to the `@farm/js-plugin-less`.See the [Less options](https://lesscss.org/usage/#less-options) for any available options you need.

### additionalData

Type:

```ts
type additionalData =
  | string
  | ((content: string, resolvePath:string) => string);
```

Default: `undefined`

Appends `Less` code to the actual entry file.
In this case, the `@farm/js-plugin-less` will not override the source but just **prepend** the entry's content.

In actual development, this becomes useful, we don't need to add new files.

> Since you're injecting code, this will break the source mappings in your entry file. Often there's a simpler solution than this, like multiple Less entry files.

#### `string`
```ts
export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
  plugins: [
    // use the less plugin.
    Less({
      additionalData: `@hoverColor: #f10215;`
    }),
  ],
});
```

#### `function`
```ts
export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
 plugins: [farmLessPlugin({
    additionalData: (content:string, resolvePath:string) => {
      if (path.basename(resolvePath,'.less') === 'index') {
        return `@hoverColor: #f10215;` + content;
      }
    },
  }) ],
});
```


### sourceMap

Type: `boolean`

Default: `false`

Whether to generate sourceMap

> If not set, it will read the compilation.sourcemap configuration in the farm configuration

### implementation

Type: `string | undefined`

Default: `undefined`

> `@farm/js-plugin-less` compatible with Less 3 and 4 versions

The special `implementation` option determines which implementation of Less to use. If you not config, it will find the less in you local node_modules.



