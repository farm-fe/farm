# Build For Production

By default, Farm has enabled support for the following features for production builds:

- **`Tree Shake`**: Crop and filter irrelevant modules and code
- **`Compression`**: Compress and mangle the output resources.
- **`Automatically inject Polyfill`**: Farm downgrades to modern browsers(ES7) by default, if you need legacy browsers support, configuring [`targetEnv`](/docs/config/compilation-options#output-targetenv)
- **`Automatic partial packaging`**: Based on dependencies and size, the project is partially bundled. For each resource request, about 25 resources are generated to ensure parallel loading performance and improve cache hits rate as much as possible.

## Configuring Output Dir

Add build script in `package.json`:

```json title="package.json" {7-8}
{
  "name": "1-create-a-project",
  "version": "1.0.0",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1",
    "start": "farm start",
    "build": "farm build",
    "preview": "farm preview"
  }
  // ...ignore other fields
}
```

Then execute `npm run build`, the built resources will be emitted to `build` dir:

```text title="Project Build Resource Structure"
build
├─ favicon.ico
├─ index.html
├─ index_02bc.bd68e90b.js
├─ index_02bc.bd68e90b.js.map
├─ index_1c74.4b50f73e.js
├─ index_7734.440d56a3.js
├─ index_880b.4631ecee.js
├─ index_8d49.63f7b906.css
├─ index_8d49.63f7b906.css.map
├─ index_9025.84e1f8e6.js
├─ index_ca37.f2c276ef.js
├─ index_ef2f.e25349d8.js
├─ index_f346.369a7312.js
```

If you want to custom the path that the resources emitted to, you can use:

- [`output.filename`](/docs/config/compilation-options#outputfilename)
- [`output.assetsFilename`](/docs/config/compilation-options#outputassetsfilename)

```ts title="farm.config.ts"
import defineConfig from "farm";

export default defineConfig({
  compilation: {
    output: {
      path: "build",
      filename: "assets/[name].[hash].[ext]",
      assetsFilename: "static/[resourceName].[ext]",
    },
  },
});
```

For above example, all `js/css` will be emitted to `build/assets/`(example: `build/assets/index-ea54.abbe3e.js`). All static assets like image will be emitted to `build/static`(example: `build/static/background.png`)

## Preview Built Resources

After the resources built, you can preview them by `npm run preview`:

```sh
$ npm run preview

> 3-build@1.0.0 preview
> farm preview

[ Farm ] Using config file at /root/tutorials/3-build-for-production/farm.config.ts
[ Farm ] preview server running at:

[ Farm ] > Local:   http://localhost:1911/
[ Farm ] > Network: http://198.18.0.1:1911/
[ Farm ] > Network: http://10.242.197.146:1911/
[ Farm ] > Network: http://192.168.1.31:1911/
```

open `http://localhost:1911/` to preview your project.

## Browser Compatibility

By default, Farm build projects to Modern Browsers that natively support `async/await`:

- Chrome >= 62
- Firefox >= 63
- Safari >= 13.1
- Edge >= 79

You can use [output.targetEnv](/docs/config/compilation-options#output-targetenv) to configuring your target browsers:

```ts
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    output: {
      targetEnv: "browser-legacy",
    },
  },
});
```

In above example, Farm will downgrade the syntax to `es5` and inject polyfill automatically. Then we have to install `core-js@3` to polyfill injection:

```sh
pnpm add -D core-js@3
```

:::note

- You need to install `core-js@3` manually if your target to legacy browsers.
- If you want to configure browsers targets more precisely, see [Syntax Downgrade And Polyfill](/docs/advanced/polyfill)
  :::

## Configure Tree Shake and Minify

Production optimization like `treeShake` and `minify` are **disabled** by default in `development` for performance reasons, and **enabled** by default in `production`. But if `treeShake` or `minify` are configured manually, the default value will be used regardless of `development` or `production`.

For details about tree shake and minify, see:

- [Tree Shake](/docs/advanced/tree-shake)
- [Minification](/docs/advanced/minification)

## Configure Partial Bundling

:::note
Refer to [Partial Bundling](/docs/advanced/partial-bundling) for details.
:::

Farm enabled best practice of bundling already, make sure you really need to configure bundles manually. See [Partial Bundling](/docs/advanced/partial-bundling) for details.
