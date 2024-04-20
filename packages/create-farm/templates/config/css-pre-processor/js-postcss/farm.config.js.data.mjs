export default function getData({ oldData }) {
  const farmSassPlugin = {
    id: "postcss",
    importer: "import postcss from '@farmfe/js-plugin-postcss'",
    initializer: "postcss()"
  }
  return {
    ...oldData,
    plugins: oldData.plugins?.flatMap((plugin) => [plugin, farmSassPlugin])
  }
}
