# Farm 命令行

`Farm Cli` 允许您启动、构建、预览和监听您的应用程序。

如果需要查看 `Farm Cli` 的可用命令, 您可以在终端中执行以下命令

```json title="Terminal"
npx farm -h
```

The output look like this:

```json title="Terminal"
farm/0.5.11

Usage:
  $ farm [root]

Commands:
  [root]
  start             启动开发服务器
  build             在生产环境下构建项目
  watch             监听文件变化并且重新构建
  preview           在本地可以直接预览您的生产环境构建出的产物
  clean [path]      清理`farm`增量构建的缓存文件
  plugin [command]  管理插件的命令

For more info, run any command with the `--help` flag:
  $ farm --help
  $ farm build --help
  $ farm watch --help
  $ farm preview --help
  $ farm clean --help
  $ farm plugin --help

Options:
  -l, --lazy           默认情况下，Farm 会在开发中延迟编译动态导入的模块，只有在模块真正执行时才会编译它们。懒惰编译确实可以加快大型项目的编译速度。
  --host <host>        host（主机）选项。它允许你指定服务器的主机地址。你可以将其设置为特定的IP地址或域名。
  --port <port>        端口）选项。它允许你指定服务器的端口号。你可以将其设置为任何未被占用的端口号。
  --open               打开）选项。它在服务器启动时自动打开浏览器。这对于快速预览你的应用程序或网站非常方便。
  --hmr                热模块替换）选项。它启用热模块替换功能，允许在运行时替换模块，而无需刷新整个页面。这对于开发过程中的实时更新非常有用。
  --cors               （跨域资源共享）选项。它启用跨域资源共享，允许从不同域的服务器请求资源。这对于开发涉及跨域请求的应用程序非常有用。
  --strictPort        （严格端口）选项。如果指定的端口已经被占用，它会导致服务器退出并显示错误消息。
  -c, --config <file>  （配置文件）选项。它允许你指定一个特定的配置文件来配置你的项目。你可以将其设置为文件的路径。
  -m, --mode <mode>    （环境模式）选项。它允许你设置项目的环境变量。环境模式可以是开发模式、生产模式或其他自定义模式。
  --base <path>        （基础路径）选项。它允许你指定公共基础路径，用于解析静态资源的相对路径。
  --clearScreen       （清除屏幕）选项。它允许你在记录日志时启用或禁用清除屏幕的功能。这对于在终端中保持日志清晰可见非常有用。
  -h, --help           显示命令帮助信息
  -v, --version        查看当前版本
```

## Start

`farm start` 命令用于启动开发服务器, 将代码进行开发环境的编译

```json title="Terminal"
Usage:
  $ farm [root]

Options:
  -l, --lazy           默认情况下，Farm 会在开发中延迟编译动态导入的模块，只有在模块真正执行时才会编译它们。懒惰编译确实可以加快大型项目的编译速度。
  --host <host>        host（主机）选项。它允许你指定服务器的主机地址。你可以将其设置为特定的IP地址或域名。
  --port <port>        端口）选项。它允许你指定服务器的端口号。你可以将其设置为任何未被占用的端口号。
  --open               打开）选项。它在服务器启动时自动打开浏览器。这对于快速预览你的应用程序或网站非常方便。
  --hmr                热模块替换）选项。它启用热模块替换功能，允许在运行时替换模块，而无需刷新整个页面。这对于开发过程中的实时更新非常有用。
  --cors               （跨域资源共享）选项。它启用跨域资源共享，允许从不同域的服务器请求资源。这对于开发涉及跨域请求的应用程序非常有用。
  --strictPort        （严格端口）选项。如果指定的端口已经被占用，它会导致服务器退出并显示错误消息。
  -c, --config <file>  （配置文件）选项。它允许你指定一个特定的配置文件来配置你的项目。你可以将其设置为文件的路径。
  -m, --mode <mode>    （环境模式）选项。它允许你设置项目的环境变量。环境模式可以是开发模式、生产模式或其他自定义模式。
  --base <path>        （基础路径）选项。它允许你指定公共基础路径，用于解析静态资源的相对路径。
  --clearScreen       （清除屏幕）选项。它允许你在记录日志时启用或禁用清除屏幕的功能。这对于在终端中保持日志清晰可见非常有用。
```

## Build

`farm build` 命令会在默认的 `dist` 目录下构建出可用于生产环境的产物。

```json title="Terminal"
Usage:
  $ farm build

Options:
  -o, --outDir <dir>    输出构建产物
  -i, --input <file>    入口文件
  -w, --watch           是否监听文件并且重新构建
  --targetEnv <target>  构建环境 node, browser
  --format <format>     构建产物格式 esm, commonjs
  --sourcemap           是否输出 sourcemap
  --treeShaking         消除无用代码而不会产生副作用
  --minify              构建时的代码压缩
  -c, --config <file>   使用指定的配置文件
  -m, --mode <mode>     设置环境模式
  --base <path>        它允许你指定公共基础路径，用于解析静态资源的相对路径。
  --clearScreen       它允许你在记录日志时启用或禁用清除屏幕的功能。这对于在终端中保持日志清晰可见非常有用。
  -h, --help            显示命令帮助信息
```

## Preview

`farm preview` 用于在本地可以直接预览您的生产环境构建出的产物, 您需要提前执行 `farm build` 来构建出生产环境的产物

```json title="Terminal"
Usage:
  $ farm preview

Options:
  --open [url]          启动时是否在浏览器中打开页面
  --port <port>         设置 Server 监听的端口号
  --host <host>         指定 Server 启动时监听的 host
  -c --config <config>  指定配置文件路径
  -h, --help            显示命令帮助
```

## Watch

`farm watch` 一般作用于 `node` 环境下监听文件变化并且重新构建

```json title="Terminal"

Usage:
  $ farm watch

Options:
  --format <format>    构建产物格式 esm, commonjs
  -o, --outDir <dir>   输出构建产物
  -i, --input <file>   入口文件
  -c, --config <file>   使用指定的配置文件
  -m, --mode <mode>     设置环境模式
  --base <path>        它允许你指定公共基础路径，用于解析静态资源的相对路径。
  --clearScreen       它允许你在记录日志时启用或禁用清除屏幕的功能。这对于在终端中保持日志清晰可见非常有用。
  -h, --help            显示命令帮助信息
```

## Clean

`farm clean` 由于 `farm` 提供的增量构建会在本地生成缓存文件, 如果在特定情况下(不可预知的编译错误)可能您需要清理缓存文件

```json title="Terminal"
Usage:
  $ farm clean [path]

Options:
  --recursive          递归搜索 `node_modules` 目录并清除缓存文件
  -c, --config <file>   使用指定的配置文件
  -m, --mode <mode>     设置环境模式
  --base <path>        它允许你指定公共基础路径，用于解析静态资源的相对路径。
  --clearScreen       它允许你在记录日志时启用或禁用清除屏幕的功能。这对于在终端中保持日志清晰可见非常有用。
  -h, --help            显示命令帮助信息
```
