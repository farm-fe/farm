import { defineConfig } from "@farmfe/core";
import less from "@farmfe/js-plugin-less"
import react from "@farmfe/plugin-react"
import reactComponents from "@farmfe/plugin-react-components"
export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    persistentCache: false,
    progress: false,
  },
  plugins: [
    less(),
    react({ runtime: "automatic" }),
    reactComponents({
      dts: "src/types/components.d.ts",
      dirs: ["src/components"],
      resolvers: [
        {
          module: "antd",
          prefix: "Ant"
        },
        {
          module: "@arco-design/web-react",
          prefix: "Arco",
          importStyle: true // style/index.js
        }
      ]
    })
  ],
});
