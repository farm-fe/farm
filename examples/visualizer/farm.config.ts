import { defineConfig } from "@farmfe/core";
// import viewer from "@farmfe/js-plugin-visualizer";
import react from "@farmfe/plugin-react";
export default defineConfig({
  plugins: [
    react(),
  ],
  compilation: {
    output: {
      // publicPath: "/aaa/",
    },
  },
  server: {
    // port: 3000,
  
  },
});
