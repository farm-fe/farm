export default function getData({ oldData }) {
  const farmPlugin = {
    id: "sass",
    importer: "import less from '@farmfe/js-plugin-sass';",
    initializer: "sass()"
  }
  return {
    ...oldData,
    plugins: oldData.plugins?.flatMap((plugin) => [plugin, farmPlugin])
  }
}
