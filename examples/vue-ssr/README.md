# Farm Vue SSR Example

Vue + Vue Router + SSR.

## Start

```sh
npm start; # start the client dev server
npm run watch; # compile and watch the server procution in development mode
```

Then visit `http://localhost:9000`.

When compiling the server-side entry, make sure to set `compilation.lazyCompilation` in the farm configuration to `false`.

```js
export default {
  input: {
    index: 'server.tsx'
  },
  compilation: {
    lazyCompilation: false
  }
};
```

## Build For Production

Build for both client and server.

```sh
npm run build && npm run build:server
```

then launch the production server:

```sh
NODE_ENV=production node server.js
```

Visit `http://localhost:3000`
