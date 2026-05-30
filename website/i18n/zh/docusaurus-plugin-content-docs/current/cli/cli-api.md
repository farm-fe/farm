# Farm 命令行

Farm CLI 用于启动开发服务器、构建生产产物、监听构建、预览已构建资源，以及清理 Farm 持久化缓存。

在安装了 `@farmfe/cli` 的项目中运行：

```bash
npx farm --help
```

## 命令

| 命令 | 作用 |
| --- | --- |
| `farm [root]` | 启动开发服务器。别名：`farm start`、`farm dev`。 |
| `farm build [root]` | 编译生产环境产物。 |
| `farm watch [root]` | 以开发模式编译，并在文件变化时重新构建。 |
| `farm preview [root]` | 本地预览已有的生产构建产物。 |
| `farm clean [path]` | 清理 Farm 持久化缓存文件。 |

## 全局选项

除命令专属帮助另有说明外，下列选项适用于所有命令：

| 选项 | 描述 |
| --- | --- |
| `-c, --config <file>` | 使用指定配置文件。默认查找 `farm.config.js`、`farm.config.ts`、`farm.config.mjs`、`farm.config.cjs`、`farm.config.mts` 或 `farm.config.cts`。 |
| `-m, --mode <mode>` | 设置配置/环境模式。 |
| `--base <path>` | 设置公共基础路径。 |
| `-d, --debug [feat]` | 输出调试日志，可选指定功能范围。 |
| `--clearScreen` | 启用或禁用终端清屏行为，默认为 `true`。 |
| `-h, --help` | 显示帮助。 |
| `-v, --version` | 显示已安装的 `@farmfe/cli` 和 `@farmfe/core` 版本。 |

## `farm [root]`、`farm dev`、`farm start`

启动 Farm 开发服务器，并以开发模式编译。

常用选项：

| 选项 | 描述 |
| --- | --- |
| `-l, --lazy` | 启用懒编译，默认为 `true`。 |
| `--host <host>` | 指定主机。 |
| `--port <port>` | 指定端口。 |
| `--open` | 启动服务器后打开浏览器。 |
| `--hmr` | 启用热模块替换。 |
| `--cors` | 启用 CORS。 |
| `--strictPort` | 请求端口被占用时退出，默认为 `true`。 |
| `--target <target>` | 设置输出目标环境：`node`、`browser` 或 `library`。 |
| `--format <format>` | 设置输出格式：`esm` 或 `commonjs`。 |
| `--sourcemap` | 生成 source map。 |
| `--treeShaking` | 启用 tree shaking。 |
| `--minify` | 启用压缩。 |

## `farm build [root]`

构建生产环境产物。

| 选项 | 描述 |
| --- | --- |
| `-o, --outDir <dir>` | 输出目录。 |
| `-i, --input <file>` | 入口 HTML/输入文件。 |
| `--target <target>` | 设置输出目标环境：`node`、`browser` 或 `library`。 |
| `--format <format>` | 设置输出格式：`esm` 或 `commonjs`。 |
| `--sourcemap` | 生成 source map。 |
| `--treeShaking` | 启用 tree shaking。 |
| `--minify` | 启用压缩。 |

## `farm watch [root]`

以开发模式构建并在文件变化时重新构建。该命令通常用于 Node 或 library 类型输出，而不是启动 Web 应用服务。

| 选项 | 描述 |
| --- | --- |
| `-o, --outDir <dir>` | 输出目录。 |
| `-i, --input <file>` | 入口输入文件。 |
| `--target <target>` | 设置输出目标环境：`node`、`browser` 或 `library`。 |
| `--format <format>` | 设置输出格式：`esm` 或 `commonjs`。 |
| `--sourcemap` | 生成 source map。 |

## `farm preview [root]`

预览已经构建完成的生产资源。请先运行 `farm build`。

| 选项 | 描述 |
| --- | --- |
| `--host [host]` | 指定主机。 |
| `--port <port>` | 指定端口。 |
| `--open` | 预览服务启动后打开浏览器。 |
| `--outDir <dir>` | 要服务的目录，默认为 `dist`。 |
| `--strictPort` | 请求端口被占用时退出。 |

## `farm clean [path]`

清理 Farm 持久化缓存文件。默认清理当前项目解析出的缓存目录。

| 选项 | 描述 |
| --- | --- |
| `--recursive` | 递归查找 `node_modules` 目录，并清理其中的 Farm 缓存位置。 |
