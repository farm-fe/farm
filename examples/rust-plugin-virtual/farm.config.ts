import { defineConfig } from '@farmfe/core';
import virtual from "@farmfe/plugin-virtual"
export default defineConfig({
  plugins: ['@farmfe/plugin-react', virtual({
    "test.js": "export default 'This is a virtual module';",
    "test1.js": {
      raw: "export const a = 1; export const b = 2; console.log(a + b);",
      moduleType: "js"
    }
  })],
});
