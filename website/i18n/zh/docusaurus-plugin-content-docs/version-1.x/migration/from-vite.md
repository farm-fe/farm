# 从 Vite 迁移

:::note
Vite 插件如 `unocss` 与 `Vite` 深度集成，由于内部设计的差异，这些插件可能与 Farm 不兼容。您可以尝试其他方法，如 `unocss postcss` 插件。
:::

从 Vite 迁移非常简单，因为 Farm 与 Vite 兼容。您需要做的就是将 `vite.config.ts` 转换为 `farm.config.ts`

- 参考[Configuring Farm](/zh/docs/config/configuring-farm) 将 farm 配置选项映射到 vite 配置
- 对于 `Vite Plugins`，将 `vite.config.ts` 中的`plugins`移动到 `farm.config.ts` 中的 `vitePlugins`

注意：

- 一些 Vite 配置选项在 Farm 中是不需要的，例如 `optimizeDeps`，您可以在迁移到 Farm 时忽略这些选项
- 对于 SSR，您需要将其重构为[Farm SSR](/zh/docs/advanced/ssr)

我们已将 [Real Vite Admin Project](https://github.com/farm-fe/farm-soybean-admin) 迁移到 Farm。有关详细信息，请查看此迁移示例
