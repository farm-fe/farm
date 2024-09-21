# Syntax Downgrade and Polyfill
By default, Farm will downgrade to `ES5` and inject `polyfills` automatically in production mode.

:::note
By default, Farm won't do transformation and inject polyfills for modules under `node_modules/`, if you need to downgrade syntax and inject polyfills for `node_modules/` you can use `compilation.presetEnv.include`.
:::

## Configuring `presetEnv`
You can use `compilation.presetEnv` to custom syntax downgrade and polyfill. Using include to add external modules that need to be polyfilled.s

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

By default, Farm will set targets to `> 0.25%, not dead`. If your project does not require browser compatibility, you can use set a looser value for `targets`, then less polyfills will be injected and output sizes will be smaller.

Refer to [compilation.presetEnv](/docs/config/farm-config#presetenv) for more options.

## With `script.target`
`script.target` can also control the target env when generate code. If you want to downgrade your project to `ES5`, you should set both:
```ts title="farm.config.ts"
export default {
   compilation: {
     script: {
      target: 'ES5'
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