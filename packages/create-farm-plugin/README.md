# create-farm-plugin

<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>Scaffolding Your First Farm Plugin Project</p>
</div>

## Scaffolding Your First Farm Plugin Project

> **Compatibility Note:**
> Farm requires [Node.js](https://nodejs.org/en/) version 16+. However, some templates require a higher Node.js version to work, please upgrade if your package manager warns about it.

With NPM:

```bash
$ npm create farm-plugin@latest
```

With Yarn:

```bash
$ yarn create farm-plugin
```

With PNPM:

```bash
$ pnpm create farm-plugin
```

Then follow the prompts!

You can also directly specify the project name and the template you want to use via additional command line options run:

```bash
# npm 6.x
npm create farm@latest my-vue-app --template js

# npm 7+, extra double-dash is needed:
npm create farm@latest my-vue-app -- --template rust

# yarn
yarn create farm my-vue-app --template js

# pnpm
pnpm create farm my-vue-app --template rust
```
