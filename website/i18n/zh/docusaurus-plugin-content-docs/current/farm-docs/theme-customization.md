# 定制 Farm Docs 主题

**预发布 API。** 使用经过验证、版本匹配的包，本页不提供尚不存在的正式安装承诺。先阅读[配置参考](./configuration.md)，需要调整表现时再扩展默认主题。Rust 负责内容、路由和编译，React 负责渲染及交互；主题不会替换内容编译器。

## 最小完整示例

```text
my-docs/
  farm.config.ts
  docs/index.md
  src/theme.tsx
  src/custom.css
```

`farm.config.ts`：

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'Example Docs',
  url: 'https://docs.example.com',
  docs: { collections: [{ id: 'main', dir: 'docs', routeBase: '/docs/' }] },
  theme: {
    components: 'src/theme.tsx',
    customCss: ['src/custom.css'],
    colorMode: 'system',
    navbar: [{ label: '指南', to: '/docs/' }]
  },
  build: { highlightRepresentation: 'static-html' }
}, { compilation: { output: { path: 'build' } } });
```

`docs/index.md`：

```md
# 欢迎

保留默认文档布局，只替换页脚。
```

`src/theme.tsx`：

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';
import { Link } from '@farmfe/docs/client';

export function Footer({ site }: ThemeProps) {
  return <footer><span>{site.title}</span><Link to="/docs/">指南</Link></footer>;
}
```

这里只替换 `Footer`，默认 `Layout`、文档页、侧栏、目录和 404 保留。将下一节 CSS 放入 `src/custom.css`，再通过已安装 CLI 执行 `farm build`、`farm preview`。仍需启用正常安装生命周期和 optional 依赖，并匹配原生包。框架先导入默认主题/高亮样式，再导入 custom CSS；不用额外引入浏览器高亮器。

## CSS tokens 与作用范围

当前默认主题使用以下变量：

| Tokens | 作用 |
| --- | --- |
| `--farm-docs-bg`、`--farm-docs-text`、`--farm-docs-muted` | 阅读背景与正文 |
| `--farm-docs-link`、`--farm-docs-border` | 强调色、链接和分隔 |
| `--farm-docs-panel`、`--farm-docs-hover` | 面板与交互背景 |
| `--farm-docs-code-bg` | 代码容器背景 |
| `--farm-docs-font-family`、`--farm-docs-font-monospace` | 界面/正文及代码字体栈 |

这是现有 CSS 的变量，不是 `SiteConfig` 字段或有独立版本保证的 token JSON API。升级时检查安装版本的真实样式。混合站点中要限制作用范围，不要直接覆盖所有 `nav`、`button`、`pre`。

```css
html:not([data-theme="dark"]) .farm-docs-theme {
  --farm-docs-bg: #faf9fc;
  --farm-docs-text: #252131;
  --farm-docs-link: #7041a8;
  --farm-docs-border: #ded7e8;
}
html[data-theme="dark"] .farm-docs-theme {
  --farm-docs-bg: #19171f;
  --farm-docs-text: #efebf6;
  --farm-docs-link: #d4b2ff;
  --farm-docs-border: #40364f;
}
.farm-docs-theme article { line-height: 1.75; }
```

示例选择器有意匹配主题的 specificity。要检查实际 computed color；只改 `:root` 可能被主题容器上的变量覆盖。保留键盘 focus、正文对比度、代码块内部滚动，以及移动端无整页横向溢出的布局。

## React 槽位与正文组件

`theme.components` 加载一个站点内模块，可导出：

| 导出 | Props / 职责 |
| --- | --- |
| `Layout` | `ThemeProps`；外层组合 |
| `DocPage`、`BlogPage` | `ThemeProps`；正文/文章 |
| `Navbar`、`Sidebar`、`Toc`、`Footer` | `ThemeProps`；独立展示区域 |
| `NotFound` | `NotFoundProps`；生成的语言相关 404 数据 |
| `default` | 仅作为 `Layout` 的回退覆盖 |
| `components` | 与默认值合并的正文组件映射 |

`ThemeProps` 包括 `site`、`page`、`children`、`data`（`home`、`pages`、`sidebars`、`blogIndexes`）、`components`、可选 `listingPosts`。`NotFoundProps` 是 `site`、带 `notFound` 的 `data`、`components`，不是伪造文章 page 的 `ThemeProps`。类型从 `@farmfe/docs/theme` 导入。

省略的槽位保留默认实现。覆盖值必须是真实 React 组件，解析器也识别 memo/forwardRef/lazy 的组件标记。不存在自动扫描 swizzle 目录的机制。

包装默认布局时转发完整 props，让其他槽位覆盖继续生效：

```tsx
import { Layout as DefaultLayout } from '@farmfe/docs-theme-default';
import type { ThemeProps } from '@farmfe/docs/theme';

export function Layout(props: ThemeProps) {
  return <DefaultLayout {...props} />;
}
```

完全自定义时，通过 `props.components` 渲染所需槽位，并转发 props/children。复用默认 CSS 时保留 `.farm-docs-theme`。只用 `{children}` 替换 Layout，可能丢失导航、搜索、文档操作区和 404 行为。

覆写正文元素时保留 ID、属性和子节点：

```tsx
import type { ComponentPropsWithoutRef } from 'react';

function Heading(props: ComponentPropsWithoutRef<'h2'>) {
  return <h2 {...props} className={`example-heading ${props.className ?? ''}`} />;
}
export const components = { h2: Heading };
```

这只修改展示，不重写已解析的标题 ID、TOC 或 Markdown 导出语义。自定义 MDX 组件仍需要实际支持的内容/导入协议；React 映射不会让 AI 导出器理解任意动态组件。

## 导航、分区侧栏和搜索

简单链接来自 `theme.navbar`。下拉菜单、图标和分区侧栏应覆盖 `Navbar`/`Sidebar`，按真实 TypeScript props 使用默认主题公开的 `NavigationMenu`、`NavigationIconLink`、`LanguageIcon`、`sidebarSection`、`filterSidebarSection`。这些工具不会向站点 JSON 增加菜单对象字段；Farm 官网的栏目/前缀数据属于站点代码，不是通用主题内置分类。

使用 `page.alternates`、生成的页面目标和 `data.sidebars`，不要靠替换 URL 前缀生成翻译/版本路由。内部导航可用 `@farmfe/docs/client` 的 `Link`、`navigate`，外链保留正常浏览器行为。菜单必须支持 hover、点击、键盘和手机触摸。

默认主题导出 `SearchProvider`、`Search`、`SearchProps`。站点可在 `DefaultLayout` 外包裹 `SearchProvider.Provider`，传入自己的 React 搜索组件。这是接入 DocSearch 的边界：仍从显式公共 Algolia 配置读取搜索参数，并由站点提供兼容的 DocSearch 依赖和样式。框架不创建密钥、爬虫或索引；有搜索界面不等于索引已覆盖 v2。

## 深浅色与代码渲染

`@farmfe/docs/client` 的 `useColorMode()` 提供 `colorMode`、`resolvedColorMode`、`setColorMode`。可选值为 `system`、`light`、`dark`。初始采用 `theme.colorMode`（通常为 system），可用的持久选择按 base path 恢复。CSS 结合 `data-theme` 和系统偏好在 hydration 前后显示对应颜色。浏览器专有逻辑放 effect 中，不能在服务端 render 时访问 `window`、`localStorage`、`matchMedia`。

Rust 同时生成代码 token 的深浅两套颜色。正文 fence 默认使用 VS Code Light 与 VS Code Dark，可通过统一的 `site.code` 选择 Farm brand 或其他预设；`.farm-code-token` 通过框架高亮样式中的 `--code-light`、`--code-dark` 切换。修改 `theme.colorMode` 只是切换显示，不会运行 JavaScript tokenizer，也不会选择任意代码主题包。

显式 `CodeBlock`、`CommandTabs` 从 `@farmfe/docs/snippets` 导入，由 Rust snippets 插件编译，并复用 `site.code`。预设包括 `farm-light`、`farm-dark`、`vscode-light`、`vscode-dark`、`inspired-github`、`base16-ocean-dark`；旧配置 `snippets.highlight.light`、`.dark` 仍可作为缺省回退，优先级参见配置参考。`snippets.rendererModule` 可指向导出 `CodeBlock`、`CommandTabs` 的展示模块。该工作流的 representation 由站点 `build.highlightRepresentation` 决定。源代码/标记必须可静态编译，不用动态表达式逃逸为运行时高亮。

`jsx` 保留生成的 token 子节点，适合需要检查它们的自定义 renderer。`static-html` 保留容器与交互，将已转义的生成 token 降为静态 HTML。覆写 `pre`/代码组件必须转发编译器的 props/属性，不要 stringify 或丢弃 HTML/token 表示。必须逐 token 访问 React children 的 renderer 应选择 jsx；不要换用 Prism、Shiki 等浏览器 tokenizer 替代 Rust 编译职责。

## Alias 与迁移边界

站点路径通过 `compilation.resolve.alias` 配置，例如用 `fileURLToPath(new URL('.', import.meta.url))` 将 `@site` 映射到站点根目录。Alias 是编译路径，不是公共 URL。用户有序 alias 会保留，但工作流为 Docs/client/runtime/theme 预留的身份具有优先权；不要覆盖它们注入另一套运行时或加载两个框架实例。

Alias 只能解析路径，不能复现 Docusaurus context、插件生命周期、主题内部接口或任意 MDX 行为。将旧导入替换为实际的 `@farmfe/docs/client`、`/mdx`、`/snippets`，小适配留站点侧。不提供包内 Docusaurus 兼容层、swizzle 命令、任意主题解析器或自动翻译服务。

验收主题时构建精确候选并实际预览，检查双主题、语言/版本链接、手机导航、搜索错误、代码复制、tabs、键盘焦点。HTML/AI 输出应保留相同路由和内容语义；正式包可用性与生产部署需要单独验证。

## 显式行内 API 签名

普通行内代码、文件路径和配置名保持原样。静态 API 签名可用 `data-language` 显式启用 Rust 编译时着色，不在浏览器运行高亮器。多行代码使用围栏；动态内容和不支持的语言会被拒绝。

```html
<code data-language="ts">(moduleId: string) =&gt; void | Promise&lt;void&gt;</code>
```


代码配色请参考配置参考中的 `site.code`：它与容器 CSS 分离，统一围栏、显式组件和静态行内签名。Farm brand 为可选预设，不要求替换主题组件。

## Farm 官网品牌配色

官网显式选择 `farm-light` / `farm-dark`，并通过 `code.tokens.light`、`code.tokens.dark` 提供品牌配色。这是站点配置方案，不会改变框架预设的兼容默认值。Rust 在编译时统一处理围栏、代码组件和显式标记的行内签名，不增加浏览器高亮器。

| Token 分类 | 浅色 | 深色 |
| --- | --- | --- |
| `foreground`、`variable` | `#292435` | `#eeecf2` |
| `keyword`、`control`、`tag` | `#8f1a7f` | `#dc96cd` |
| `type`、`class`、`attribute` | `#526b87` | `#a8b9ce` |
| `function` | `#497774` | `#9fbebc` |
| `parameter` | `#5f596b` | `#c7bfd1` |
| `property` | `#676071` | `#b8b0c2` |
| `string`、`number`、`constant` | `#8a6e42` | `#c8b391` |
| `comment`、`punctuation` | `#746d7a` | `#9e99a4` |

将每一分类分别写入对应模式的 token map，即可复用官网配色。代码容器背景（`#faf7fb` / `#211d25`）、间距和圆角仍由 CSS 定制；默认样式直接显示配置的 token 颜色，不再叠加第二套 CSS 混色。

官方 Mermaid 流程图使用配对的明暗静态 SVG，保留可维护的 Mermaid 源码。可选的作者渲染步骤与 `farm build` 分离，读者不加载 Mermaid 运行时。替换图表视觉时，应保留可访问说明、源码链接及真实条件分支。
