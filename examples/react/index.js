import {
  createCompiler,
  createDevServer,
  createFileWatcher,
  resolveConfig,
  start,
} from "@farmfe/core";

const resolvedUserConfig = await resolveConfig({
  compilation: {
    sourcemap: true,
    persistentCache: false,
    presetEnv: false,
    progress: false,
    output: {
      publicPath: '/dist/'
    },
    input: {
      index: './index.html'
    }
  },
  server: {
    port: 6532,
    hmr: {
      path: '/__farm_hmr'
    }
  },
  plugins: [
    '@farmfe/plugin-react',
    '@farmfe/plugin-sass'
  ],
  mode: 'development',
});

const compiler = await createCompiler(resolvedUserConfig);

const devServer = await createDevServer(compiler, resolvedUserConfig);

await devServer.listen();

await start({
  compilation: {
    sourcemap: true,
    persistentCache: false,
    presetEnv: false,
    progress: false,
    output: {
      publicPath: '/dist/'
    },
    input: {
      index: './index.html'
    }
  },
  server: {
    port: 6532,
    hmr: {
      path: '/__farm_hmr'
    }
  },
  plugins: [
    '@farmfe/plugin-react',
    '@farmfe/plugin-sass'
  ],
  mode: 'development',
});
