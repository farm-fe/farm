# Farm Commerce Lab

A deterministic merchant operations application with JavaScript, Rust and mixed plugin pipelines. See the [functional acceptance guide](../../docs/experiences/commerce-functional-acceptance.md) for safe commands and the outstanding native/browser gates.

## 开发和验证

在仓库根目录安装 workspace 依赖并按构建技能重建当前源码的 core 和插件。已有旧 `.node` / `.farm` 文件不能证明与当前源码匹配。

在本目录执行：

```powershell
npm test
$env:COMMERCE_GROUP='js' # js / rust / mixed
npm run build
npm start
```

另一终端运行端到端业务验收：

```powershell
node tests/browser.mjs
```

端口默认 4310，可通过 `COMMERCE_PORT` 修改。验收目标通过 `COMMERCE_URL` 修改。每组输出保存在 `dist/<group>`，缓存保存在 `.cache/<group>/<cacheId>`，`COMMERCE_CACHE_ID` 默认 `default`。

冷构建使用未使用过的 cacheId，缓存构建使用同一 cacheId 连续运行。Worker 产物已有缓存构建保留与 gzip 回解验证记录；最新证据见 `docs/v2-delivery-status.md`。使用新 cacheId 的构建成功不能替代缓存构建验收。

## 业务与对照范围

数据导入支持 CSV 文件上传和逐行字段修正。修正后必须重新运行 Worker 校验，重复 SKU、非法价格和格式错误会阻止导入；通过后创建草稿商品，再在商品页发布。

全部行校验通过后，可生成和下载给集成团队使用的 TypeScript 导入数据接口。实际运行固定版本 `json_typegen_wasm@0.7.0` 的 WASM，包含全部校验行；CSV 值保持字符串，生成结果不替代运行时校验。导入完成、数据变更和商家切换会清除旧预览。JS 配置使用公开的薄 Farm WASM 适配器，以 Node WebAssembly API 读取模块信息；Rust 与 mixed 使用 `@farmfe/plugin-wasm`，浏览器执行同一二进制。此项是接入方式对比，不宣称 WASM 算法的 JS/Rust 速度差异，也不覆盖任意 WASM 模块、SSR 或 `?init` API。

商品编辑支持名称、美元价格与库存数量，拒绝无效数值并保留原数据；保存后持久化到本地 demo 状态。管理员可跨搜索结果选择商品并批量发布，操作在 domain 中一次性校验商家归属。

点击订单 ID 查看关联商品、SKU、客户、数量、订单金额、渠道与状态；发货或退款后详情同步更新。viewer 只能读取详情，商家切换清理选中订单。退款保留历史账单金额，经营指标在 domain 层排除已退款订单。

订单中心使用固定 48px 行高的滚动虚拟表格。`createFixtures(42, { productsPerTenant: 120, ordersPerTenant: 10000 })` 可生成两个商家共 20,000 条订单，保持商品归属、金额和稳定 ID；默认 demo 仍是每商家 60 商品 / 400 订单。

在本目录执行 `node tests/orders-browser.mjs` 自动依次构建并预览 JS、Rust、混合三组，在独立浏览器上下文中安装上述规模的业务数据，检查 DOM 行数有界、实际行高、末尾滚动、发货、商家与筛选重置。该脚本占用 4352 端口并关闭自己启动的预览服务，原始报告保存到 `reports/orders-browser-*.json`。这属于正确性验收，不是构建性能数据或编译模块规模扩展。

演示商家隔离、概览、商品、订单、CSV Worker 校验、活动发布、MDX 帮助及组件实验室。价格使用整数美分，fixtures 固定 seed 42，Reset demo 恢复基线。

- JS：官方 JS SVGR/Sass/Tailwind、unplugin 自动导入、成熟数据转换器的薄 Farm load 接入。
- Rust：对应 Rust 插件。
- 混合：保留 JS 数据转换，其他能力走 Rust。
- React 与 Worker 目前三组共用，不能计入 JS/Rust 替换收益。
- 已启用三组 gzip 压缩和独立 JS/Rust DTS SDK 子实验。最终 native 的 SDK ESM 打包消费、热缓存及类型更新/恢复验收通过；Vue SFC/JSX 的 JS/Rust 四组合生产业务、严格类型及开发更新验收通过。按需组件等其余插件仍需完成。

Rust DSV 已修复为对象行导出，并增加独立回归与浏览器导入验证。不得将构建成功作为业务验收成功。

## SDK 缓存与外部消费验收

先按仓库构建流程生成 core、JS DTS 与 Rust DTS 插件产物，然后在本目录运行：

```powershell
node tests/sdk-cache.mjs
node tests/sdk-type-update.mjs
```

脚本默认使用新缓存 ID，在相同缓存上执行首次、warm、warm-repeat 三轮 JS/Rust SDK 构建。每轮通过 npm pack 和外部临时目录安装，检查完整类型依赖、严格 NodeNext 类型消费及运行时金额计算。原始构建和消费记录保存在 `reports/sdk-cache-*.json`。可设置 `COMMERCE_SDK_CACHE_ID` 复用指定缓存；脚本报告只证明功能验收，不作为性能统计样本。

`sdk-type-update.mjs` 使用同一缓存验证仅类型模型的修改、再次复用、恢复和恢复后复用，每轮检查声明文本并进行外部消费。源码在 finally 中恢复，报告保存在 `reports/sdk-type-update-*.json`。验收期间保持这些 SDK 源文件不被其他进程编辑。

## Vue JSX 独立实验

`apps/vue-jsx-lab/` 复用库存业务和 `tests/vue-browser.mjs`。分别使用 `farm.vue-jsx-js.config.ts`、`farm.vue-jsx-rust.config.ts` 构建/预览，设置 `COMMERCE_VUE_VARIANT=jsx` 和对应 group、URL 执行业务验收。

`pnpm run type-check:vue-jsx` 检查 Vue JSX，设置 variant 后执行 `node tests/vue-types.mjs` 验证错误组件属性探针。两组生产业务及脚本 HMR 已通过，Rust 共享 domain HMR 也已通过。`node tests/vue-script-hmr.mjs`、`node tests/vue-dependency-hmr.mjs` 支持 JSX variant，测试编辑并恢复源码，不要并行执行源码编辑测试。全部插件选项及正式性能采样仍需继续验证。

## 客户档案

客户中心按商家展示复购档案，净支出排除退款，退款记录和历史订单仍保留。搜索客户后可以展开档案、分页查看订单并打开关联商品详情；切换商家会清空选中客户和订单。fixture 使用固定客户循环，保证复购记录包含不同渠道与订单状态。viewer 可以阅读这些详情，不能执行订单退款。

## 活动集合发布

Campaign rules 页允许管理员设置当前商户的 1–80% 整数折扣上限，保存后刷新数据仍保留；非法输入不改变规则，viewer 仅查看。活动创建和报价预览使用相同上限，已发布活动不因后续降低上限而改价。价格表按十件分页展示完整所选集合，汇总每个商品各一件的原价、报价与节省，不作为销售预测。

营销活动可以搜索并选择已发布商品集合，折扣预览只计算该集合。先上传 CSV、修正校验错误并发布商品，再在 Campaigns 选择新商品即可完成演示链路。空集合、重复商品、小数折扣与跨商家商品不能发布；活动持久化在本地，viewer 无权访问发布页面。浏览器验收也覆盖活动金额预览、商家隔离和刷新后的持久化。

## 经营概览真实图表

渠道收入环形图使用同一日期区间和渠道筛选，配套明细表显示三个渠道的金额与订单数。切换单渠道时其余渠道归零，零收入区间显示明确提示。验收以独立 fixture 计算逐渠道预期，截图保存为 `reports/channel-chart-*.png`。

独立 Channels 页面复用这些统计和图表组件，可以聚焦单个渠道，查看收入趋势与平均订单金额；日期范围和商家切换也受浏览器验收覆盖。平均订单金额以整数美分舍入，退款订单不进入分子或分母。

经营概览精确固定 ECharts 6.1.0，按需加载柱图、坐标网格、提示框和 SVG renderer。日期范围与渠道筛选同时更新收入、订单和每日图表，退款不计入收入；展开每日数据表可查看相同口径的整数美分金额。图表在容器尺寸变化时 resize，在路由离开时 dispose。

`node tests/orders-browser.mjs` 顺序构建并验证 JS / Rust / mixed，覆盖图表实际 SVG、筛选口径和零订单区间，同时运行订单、商品和 CSV 完整业务回归。`COMMERCE_CACHE_ID` 可以指定独立缓存；默认缓存和全新缓存均已通过，原始报告保存在 `reports/orders-browser-*.json`。这些结果证明当前生产业务正确性，正式性能报告需按完整测量协议另行采样。

## 库存与帮助中心

Products 表格显示分类 SVG 插图缩略图，并提供可缩放的图片预览。`?url` 资源经真实资产输出加载；图片注明为本地演示分类插图。Viewer 可预览，切换商户清除选择。浏览器测试核对输出资源加载、缩放、关闭与商户隔离。

Inventory 页支持 SKU/名称搜索、低库存筛选、商户库存总量与按当前售价估值。管理员输入明确的正整数收货数量，非法输入不改变库存；未提交收货在商户切换时清除，已确认收货刷新后保留。Viewer 仅查看库存。

收货确认前显示当前、新增及最终库存数量，预览与确认共用安全整数/溢出校验。Connection 页可选择商品、订单、指标和活动资源，模拟健康、408 超时及 403 拒绝；失败/重试清除旧结果，切换商户或角色取消旧请求。Viewer 的活动请求被拒绝，商品和订单可读取；指标金额不包含退款订单。

Help center 按指南标题与主题关键词检索，导航到导入、订单操作或活动发布的实际 MDX 文档。订单指南说明流转和退款口径，活动指南提供 YAML 规则示例。浏览器验收覆盖检索、空结果、指南切换和代码块；检索范围不包含全文。

## 实际编译图规模

数据导入支持上传或编辑 CSV、Worker 校验及逐行纠错。纠错表每页 10 行，显示原始 CSV 行号，跨页切换保留修改；修改后必须重新校验才可导入。浏览器验收使用 25 条商品，在末页修正价格错误后核对整批草稿、整数分价格与商户归属，并复验 Worker/文件读取失败、取消及过期响应处理。

`node benchmark/component-inventory.mjs` 从主 React 入口沿本地静态/动态导入审计 TS/TSX，列出静态 JSX 引用的手写业务/共享组件，页面与基础包装分开。生成扩展、MDX/SVG 转换输出、第三方依赖及未引用导出不计入目标规模。报告包含源码身份；静态引用不能代替浏览器行为覆盖或真实编译图。

扩展负载先运行 `node benchmark/extension-generator.mjs baseline`、`medium` 或 `large`，分别生成 4／32／128 分类表单及真实 SVG、Sass、MDX 依赖。设置 `COMMERCE_SCALE=medium` 或 `large` 后构建或运行图快照；默认 baseline。Components 页面可以选择分类、编辑并保存属性。生成器只写入该档目录，同名文件内容不同会拒绝覆盖；生成负载单独报告，不计入人工业务组件规模。

`node benchmark/graph-snapshot.mjs` 记录当前组主编译图，使用独立缓存/输出。设置 `COMMERCE_GROUP=rust` 或 `mixed` 切换配置，工具断言实际配置文件匹配请求并保存插件身份。报告 `reports/graph-*.json` 包含保留模块、边、资源字节以及完整图和 provenance；已移除端点的 trace 边单独计数。资源统计包含 sourcemap/gzip，主图不代表 Worker 子图；订单条数与源码文件数均不能替代真实组件规模。

排查挂起时设置 `COMMERCE_TRACE_HOOKS=1`，工具每秒写入同 identity 的 `.live.json`，记录阶段、pending hook、进程身份；完成后写 terminal=true 与实际结果。若进程被外部结束，日志不会自动变成终止证据，必须核对进程是否仍存在。诊断会包装 hook，报告明确标记 diagnosticTracing，不使用该模式的时间或内存作正式性能结论。

开发启动停滞可单独运行 `node benchmark/startup-diagnostics.mjs`，默认 mixed；用 `COMMERCE_GROUP=js` 或 `rust` 切换，用 `COMMERCE_CACHE_ID` 指定要观察的组缓存。工具执行真实 Server 启动/关闭，在原生 Compiler 创建前包装 JS hooks，每秒记录 compiling、pending plugin/hook/path 和 PID。报告带源码/native 身份和 diagnosticTracing，结束时写 terminal=true；外部结束后仍应核对 PID。诊断插件会影响调度和缓存身份，连续通过不能代替普通 HMR 验收或证明偶发停滞已消除。运行时避免同时编辑 workload 或原生产物。

## Vue 独立实验

`apps/vue-lab/` 复用主应用 domain 的 seed 42 商家、商品和库存规则。两组共享 SFC、搜索、补货、角色权限、商家隔离及 scoped 样式；仅配置替换 Vue 编译路径。

```powershell
node ../../packages/cli/bin/farm.mjs build --config farm.vue-js.config.ts
$env:COMMERCE_PORT='4330'
node ../../packages/cli/bin/farm.mjs preview --config farm.vue-js.config.ts
# 另一终端
$env:COMMERCE_GROUP='js'
$env:COMMERCE_URL='http://localhost:4330'
node tests/vue-browser.mjs
```

Rust 组使用 `farm.vue-rust.config.ts` 和 `COMMERCE_GROUP=rust`。HMR 验证需要运行对应配置的 `farm start`，然后设置目标地址执行 `node tests/vue-hmr.mjs`；测试修改并恢复库存组件，检查模板与样式更新是否保留业务状态。`node tests/vue-script-hmr.mjs` 检查脚本变化重载组件、新逻辑生效及源码恢复。不要同时运行任何会编辑 SFC 的测试。

`pnpm run type-check:vue` 使用 Vue 3.4.35、TypeScript 5.4.5 和 vue-tsc 2.0.29 检查真实 SFC。`node tests/vue-types.mjs` 执行基线、错误组件属性探针、源码恢复三阶段检查，确认类型检查确实覆盖模板属性；该测试也会编辑并恢复源码，应在开发服务器停止后单独执行。

当前 JS 路径锁定 Vue 3.4.35 和 Vite Vue 插件 5.1.4。最终 native 下 SFC/JSX 的 JS/Rust 四组合生产库存业务、脚本更新和外部依赖更新通过，SFC 的模板/native scoped CSS 状态保留通过。两种入口的严格类型检查均实际拒绝错误组件属性并在源码恢复后通过。聚合报告是 `reports/vue-final-1791768982451/run.json`，最终验收证据见 `../../docs/release-evidence/v2-final-vue-sdk-acceptance.json`。正式插件包、其他平台、更多编译边界及统一性能采样仍需验收，不发布 Vue 性能排名。
