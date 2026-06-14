# Syntax Downgrade and Polyfill
Farm can downgrade syntax and inject polyfills automatically in production mode when you choose a versioned browser [`output.targetEnv`](/docs/config/compilation-options#output-targetenv), such as `browser-es2017`, `browser-es2015`, or `browser-legacy`.

:::note
By default, Farm won't do transformation and inject polyfills for modules under `node_modules/`, if you need to downgrade syntax and inject polyfills for `node_modules/` you can use `compilation.presetEnv.include`.
:::

## Configuring `targetEnv`
Farm provides a normalized [`output.targetEnv`](/docs/config/compilation-options#output-targetenv) option to configure the target execution environment of your application. Versioned browser targets enable matching `syntax downgrade` and `polyfill injection` automatically. For example:

```ts title="farm.config.ts"
export default {
  compilation: {
    output: {
      targetEnv: 'browser-legacy'
    }
  },
};
```

Farm will compile your application to legacy browsers(ES5):
* Compile all `Js/Jsx/Ts/Tsx` modules to `ES5`, and inject all polyfills(Promise, regenerator-runtime and so on).

Farm supports normalized `targetEnv` options like `browser-esnext`, `browser-es2017`, `browser-es2015`, `browser-legacy`, `node16`, `node-legacy`, and `node-next`. Set `targetEnv` explicitly when you need a specific compatibility target. Refer to [`output.targetEnv`](/docs/config/compilation-options#output-targetenv).

:::note
You may need to install `core-js@3` or `regeneration-runtime` manually if polyfill is needed. Try run `pnpm add core-js` if you met something error like `can not resolve 'core-js/modules/xxx'`
:::

## Configuring Syntax and Polyfill Separately
Internally, `targetEnv` selects production compatibility defaults such as `presetEnv` and, for versioned browser targets, `script.target`. You can configure these options more precisely if you need.

### Configuring `presetEnv`
You can use `compilation.presetEnv` to custom syntax downgrade and polyfill injection. By default all modules under `node_modules` will be ignored. Using `include` to add extra modules that need to be polyfilled.

```ts title="farm.config.ts"
export default {
   compilation: {
     presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: "Chrome >= 48"
      }
     }
   },
};
```

Note that if your project does not require browser compatibility, you can use set a looser value for `targets`, then less polyfills will be injected and output sizes will be smaller.

Refer to [`compilation.presetEnv`](/docs/config/compilation-options#presetenv) for more options.

### Configuring `script.target`
`script.target` is used to control the target env when generate code. If you want to downgrade your project to `ES5`, you should set both:

```ts title="farm.config.ts"
export default {
   compilation: {
     script: {
      target: 'es5'
     },
     presetEnv: {
      // include a package under node_modules
      include: ['node_modules/package-name'],
      options: {
        targets: "> 0.25%, not dead"
      }
     }
   },
};
```