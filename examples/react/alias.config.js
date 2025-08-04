import { defineConfig } from "farm";

export default defineConfig(() => {
  // console.log(__dirname);
  // console.log(__filename);
  // console.log(__dirname);

  return {
    root: "./react",
    compilation: {
      sourcemap: false,
      persistentCache: true,
      presetEnv: false,
      progress: false,
      output: {
        publicPath: "/dist/",
      },
    },
    server: {
      port: 9652,
      hmr: {
        path: "/__farm_hmr",
      },
    },
    plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass"],
  };
});
