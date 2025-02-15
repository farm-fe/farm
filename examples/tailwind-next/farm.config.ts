import { defineConfig } from "@farmfe/core";
import tailwind from "@farmfe/js-plugin-tailwindcss";

export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  server: {
    writeToDisk: true,
  },
  plugins: [
    "@farmfe/plugin-react",
    tailwind()
  ],
});
