import type { JsPlugin } from "@farmfe/core";
import { babel, type BabelOptions } from "@farmfe/js-plugin-babel";
import type { PluginOptions } from "./types";

interface ReactCompilerOptions
  extends Pick<BabelOptions, "filters" | "transformOptions"> {
  compilerOptions?: Partial<PluginOptions>;
}

const defaultFilters: ReactCompilerOptions["filters"] = {
  moduleTypes: ["jsx", "tsx"],
  resolvedPaths: [],
};

export function reactCompiler(options: ReactCompilerOptions = {}): JsPlugin {
  return babel({
    name: "js-plugin:react-compiler",
    priority: 120,
    filters: {
      moduleTypes: options.filters?.moduleTypes ?? defaultFilters.moduleTypes,
      resolvedPaths:
        options.filters?.resolvedPaths ?? defaultFilters.resolvedPaths,
    },
    transformOptions: {
      plugins: ["babel-plugin-react-compiler", "@babel/plugin-syntax-jsx"].map(
        (pkg) => require.resolve(pkg)
      ),
    },
  });
}
