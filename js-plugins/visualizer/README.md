<div align="center">
  <a href="https://github.com/farm-fe/farm">
    <img src="../../assets/logo.png" width="550" />
  </a>
</div>

---

# Visualizer Plugin for Farm

## Getting Started

To begin, you'll need to install `@farmfe/visualizer`:

```console
npm install @farmfe/visualizer --save-dev
```

or

```console
yarn add -D @farmfe/visualizer
```

or

```console
pnpm add -D @farmfe/visualizer
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from 'farm/dist/config';
import visualizer from '@farmfe/visualizer'; //  import the plugin

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  plugins: [
    // use the visualizer plugin.
    visualizer({
      // custom options here
    })
  ]
});
```

## Options

- **[`RecordViewerOptions`](#RecordViewerOptions)**

### RecordViewerOptions

Type:

```ts
type RecordViewerOptions = {
  /**
   * Specify hostname
   * @default '127.0.0.1'
   */
  host?: string;

  /**
   * Specify port
   * @default 9527
   */
  port?: number;
}
```

Default: undefined

## Credits

Thanks to:

- The [vite-plugin-inspect](https://github.com/antfu/vite-plugin-inspect) project created by [Anthony Fu](https://github.com/antfu), inspiring the module analysis feature in Farm's Visualizer.

- The [rsdoctor](https://github.com/web-infra-dev/rsdoctor) project created by [web-infra](https://github.com/web-infra-dev), influencing the design of Farm's Visualizer.