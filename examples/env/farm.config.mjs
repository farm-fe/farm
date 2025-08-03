import { defineConfig } from "farm";
export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  envPrefix: ["FARM_", "CUSTOM_PREFIX_", "NEW_"],
});
