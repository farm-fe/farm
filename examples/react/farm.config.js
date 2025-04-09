import { defineConfig } from "@farmfe/core";
import react from "@farmfe/plugin-react";

export default defineConfig((env) => {
  console.log(env);
  console.log(process.env.NODE_ENV);

  return {
    compilation: {
      sourcemap: true,
      persistentCache: false,
      presetEnv: false,
      minify: false,
      progress: false,
      // output: {
      //   publicPath: '/dist/'
      // },
      runtime: {
        isolate: true,
      },
    },
    server: {
      port: 4000,
      proxy: {
        "^/(api|login|register|messages)": {
          target: "https://petstore.swagger.io/v2",
          ws: true,
        },
      },
    },
    plugins: [
      react({
        useAbsolutePath: true,
      }),
      "@farmfe/plugin-sass",
      [
        "@farmfe/plugin-virtual",
        {
          "virtual-module": "export const a = 1",
          "src/01.js": 'export const module01 = "virtual-module"',
        },
      ],
    ],
  };
});
