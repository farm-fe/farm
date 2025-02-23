const styleXPlugin = require("@stylexjs/babel-plugin");
const stylexExtendPlugin = require("@stylex-extend/babel-plugin");

module.exports = {
  plugins: {
    "@stylexjs/postcss-plugin": {
      include: [
        "src/client/pages/**/*.{js,jsx,ts,tsx}",
        "src/client/themes/**/*.{js,jsx,ts,tsx}",
        "src/client/components/**/*.{js,jsx,ts,tsx}",
      ],
      babelConfig: {
        parserOpts: {
          plugins: ["jsx", "typescript"],
        },
        plugins: [
          [
            stylexExtendPlugin,
            {
              unstable_moduleResolution: {
                type: "commonJS",
                rootDir: __dirname,
              },
              transport: "props",
            },
          ],
          [
            styleXPlugin,
            {
              runtimeInjection: false,
              dev: false,
              // Required for CSS variable support
              unstable_moduleResolution: {
                type: "commonJS",
                rootDir: __dirname,
              },
            },
          ],
        ],
      },
    },
    autoprefixer: {},
  },
};
