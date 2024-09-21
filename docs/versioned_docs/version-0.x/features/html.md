---
sidebar_position: 1
---

# Html

## Basic Usage

Farm support compile Html out of box, **and you should use Html as entry when build a web project**, for example:

```ts title="farm.config.ts"
import type { defineConfig } from "@farmfe/core";

export default defineConfig({
  input: {
    index: "./index.html", // using ./index.html as entry
  },
});
```

:::note
If the input is not specified, default to `{ index: 'index.html' }`.
:::

and in `./index.html`, a `<script src="./xxx">` should be used to refer to your script entry.

```html title="./index.html"
<html>
  <!-- ... -->
  <body>
    <div id="root"></div>
    <!-- index.ts is the script entry -->
    <script src="./index.ts"></script>
  </body>
</html>
```

and you can also use `<link href="./xxx">` to refer to your global css.

Farm will transform these `scripts` and `links` to final production resources when compiling. Note that you have to use `relative path` when you want to refer to a local module, for example `<script src="./index.tsx"></script>` will refer to a local module and compile it, but `<script src="/index.tsx"></script>` or `<script src="https://xxx.com/index.tsx"></script>` would not.

:::tip
The `script` and `link` can refer to any module types that farm support, for example, `js`, `jsx`, `ts`, `tsx`, or other module types supported by plugins. You can use as many `scripts` or `links` as you want.
:::

## Multi Page App

If you are building a Multi Page Application, just configure multiple html input, for example:

```ts title="farm.config.ts"
import type { UserConfig } from "@farmfe/core";

export function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      home: "./index.html", // Home Page
      about: "./about.html", // About Page
      // ... more pages
    },
  },
});
```

Farm will compile these pages in parallel, and all dependencies of these pages will be shared too.

## Inherit html template

Farm supports inherit html template by using `html.base` config, which is helpful when building a multi-page application with html shared.

```ts title="farm.config.ts"
import type { UserConfig } from "@farmfe/core";

export function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // ...
  compilation: {
    input: {
      home: "./index.html", // Home Page
      about: "./about.html", // About Page
      // ... more pages
    },
    // c-highlight-start
    html: {
      base: "./base.html",
    },
    // c-highlight-end
  },
});
```

Then add a `base.html`, placeholder `{{children}}` will be replaced by children's content.

```html title="./base.html"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body>
    <div id="root"></div>
    <!-- using children placeholder and it will be replaced -->
    {{children}}
  </body>
</html>
```

Inherit `./base.html`:

```html title="./src/home.html"
<!-- Other fields are inherit from ../base.html -->
<script src="./index.tsx"></script>
```
