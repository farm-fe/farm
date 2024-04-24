export default function getData({ oldData }) {
  const farmPlugin = {
    id: "less",
    importer: "import less from '@farmfe/js-plugin-less';",
    initializer: "less()"
  }
  return {
    ...oldData,
    plugins: oldData.plugins?.flatMap((plugin) => [plugin, farmPlugin])
  }
}
