import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
export default defineConfig({
  plugins: [react()],
  server: {
    // port: 3005,
  },
});
