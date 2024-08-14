import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
export default defineConfig({
  plugins: [react()],
  compilation: {
    output: {
      publicPath: '/aaa/',
    }
  }
});
