# CLI 选项

## create

创建一个新的Farm项目。

```bash
pnpm create farm
# 或 npm create farm
# 或 yarn create farm
# choose your favorite package manager
```

其他命令由包 `farm` 提供：

## start

启动开发服务器，在开发模式下编译 Farm 项目并监视文件更改。

```bash
farm start
```

## build

以生产模式构建 Farm 项目

```bash
farm build
```

## preview

预览 `build` 命令的结果。

```bash
farm build && farm preview
```

## watch

Watch 通常用于编译库项目，它的工作方式类似于 `start` 命令，但它不会启动开发服务器。

```bash
farm watch
```
