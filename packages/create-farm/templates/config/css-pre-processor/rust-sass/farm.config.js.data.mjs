export default function getData({ oldData }) {
  const farmPlugin = {
    initializer: "'@farmfe/plugin-sass'"
  }
  return {
    ...oldData,
    plugins: oldData.plugins?.flatMap((plugin) => [plugin, farmPlugin])
  }
}
