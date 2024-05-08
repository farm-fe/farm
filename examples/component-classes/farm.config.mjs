import { defineConfig } from "@farmfe/core";

export default defineConfig({
	compilation: {
		input: {
			index: "./index.js",
		},
		presetEnv: {
			options: {
				targets: {
					esmodules: true,
				},
			},
		},
		minify: false,
		output: {
			entryFilename: "[entryName].mjs",
		},
	},
});
