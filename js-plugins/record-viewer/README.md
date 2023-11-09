<div align="center">
  <a href="https://github.com/farm-fe/farm">
    <img src="../../assets/logo.png" width="550" />
  </a>
</div>

---

# Reocrd Viewer Plugin for Farm

## Getting Started

To begin, you'll need to install `@farmfe/js-plugin-record-viewer`:

```console
npm install @farmfe/js-plugin-record-viewer --save-dev
```

or

```console
yarn add -D @farmfe/js-plugin-record-viewer
```

or

```console
pnpm add -D @farmfe/js-plugin-record-viewer
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';
import record from '@farmfe/js-plugin-record-viewer'; //  import the plugin

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
    // use the record viewer plugin.
    record({
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



