# Overview

Farm officially provides a lot of useful plugins, including Rust plugins and JS plugins. Rust plugins are much faster than Js plugins, we recommend to use Rust plugins whenever possible.

:::tip
Refer to [Using Plugins](/docs/using-plugins) for how to use plugins in Farm.
:::

## Rust Plugins

* **[`@farmfe/plugin-react`](./react)**：Support React `jsx` and `react-refresh`.
* **[`@farmfe/plugin-sass`](./sass)**：Support compiling `sass/scss` files.
* **[`@farmfe/plugin-strip`](./strip)**：A Farm rust plugin to remove `debugger` statements and functions like `assert.equal` and `console.log` from your code.
* **[`@farmfe/plugin-dsv`](./dsv)**：A Farm plugin which converts `.csv` and `.tsv` files into JavaScript modules.
* **[`@farmfe/plugin-yaml`](./yaml)**：A Farm plugin which Converts YAML files to ES6 modules.
* **[`@farmfe/plugin-virtual`](./virtual)**：A rust plugin for farm to easily use virtual module.
* **[`@farmfe/plugin-react-components`](./react-components)**：On-demand components auto importing for React.

## Js Plugins

* **[`@farmfe/js-plugin-postcss`](./js-postcss)**：Support `postcss` in your project.
* **[`@farmfe/js-plugin-less`](./js-less)**：Support compiling `less` files.
* **[`@farmfe/js-plugin-svgr`](./js-svgr)**：Support compiling `svg` files.
* **[`@farmfe/js-plugin-dts`](./js-dts)**：Support compiling `*.d.ts` files.
* **[`@farmfe/js-plugin-sass`](./js-sass)**：Support compiling `sass/scss` files.

## Community Plugins

If official plugins doesn't meet your needs, you can try [Community Plugins](../community-plugins)

And of course check out [awesome-farm](https://github.com/farm-fe/awesome-farm) - you can also submit a PR to list your plugins there.
