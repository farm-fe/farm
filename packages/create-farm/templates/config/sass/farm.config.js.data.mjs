export default function getData({ oldData }) {
  const farmSassPlugin = {
    initializer: "'@farmfe/plugin-sass'"
  }
  return {
    ...oldData,
    plugins: oldData.plugins?.flatMap((plugin) => [plugin, farmSassPlugin])
  }
}
