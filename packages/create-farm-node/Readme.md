# create-farm

<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>Scaffolding Your First Farm Project</p>
</div>

This package provides official templates for the Rust Web Bundler [Farm](https://github.com/farm-fe/farm).

## Scaffolding Your First Farm Project

> **Compatibility Note:**
> Farm requires [Node.js](https://nodejs.org/en/) version 16+. However, some templates require a higher Node.js version to work, please upgrade if your package manager warns about it.

With NPM:

```bash
$ npm create farm@latest
```

With Yarn:

```bash
$ yarn create farm
```

With PNPM:

```bash
$ pnpm create farm
```

Then follow the prompts!

You can also directly specify the project name and the template you want to use via additional command line options run:

```bash
# npm 6.x
npm create farm@latest my-vue-app --template react

# npm 7+, extra double-dash is needed:
npm create farm@latest my-vue-app -- --template vue

# yarn
yarn create farm my-vue-app --template react

# pnpm
pnpm create farm my-vue-app --template vue
```

Inspiration comes from [create-tauri-app](https://github.com/tauri-apps/create-tauri-app/#usage)
