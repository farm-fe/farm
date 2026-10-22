import type { ReactNode } from 'react';
import { Children, isValidElement } from 'react';
import { CodeBlock, type CodeBlockProps } from './code-block.js';
import { TabItem, Tabs, type TabsStorageMigration } from './content-tabs.js';
import type { CommandTool } from './snippets.js';

export { CodeBlock, type CodeBlockProps };
export interface CommandTabsProps { children?: ReactNode; storageMigration?: TabsStorageMigration; labels?: Partial<Record<CommandTool, ReactNode>> }
interface CompiledPanelProps extends CodeBlockProps { 'data-command-tool': CommandTool }

/** Present Rust-compiled panels; all panels remain readable in SSR without JS. */
export function CommandTabs({ children, storageMigration, labels }: CommandTabsProps) {
  const seen = new Set<string>();
  const panels = Children.toArray(children).filter(child => !(typeof child === 'string' && !child.trim())).map(child => {
    if (!isValidElement<CompiledPanelProps>(child) || child.type !== 'pre') throw new Error('[DOCS_SNIPPET_PROTOCOL] expected compiled pre panels');
    const tool = child.props['data-command-tool'];
    if (!['npm', 'pnpm', 'yarn', 'bun'].includes(tool) || seen.has(tool) || typeof child.props['data-code'] !== 'string') throw new Error('[DOCS_SNIPPET_PROTOCOL] expected unique tools and compiled code');
    seen.add(tool);
    return child.props;
  });
  if (!panels.length) throw new Error('[DOCS_SNIPPET_PROTOCOL] expected at least one compiled command panel');
  return <Tabs label="Package manager" groupId="farm-docs-package-manager" storageMigration={storageMigration}>
    {panels.map(panel => <TabItem key={panel['data-command-tool']} value={panel['data-command-tool']} label={labels?.[panel['data-command-tool']] ?? panel['data-command-tool']}><CodeBlock {...panel}/></TabItem>)}
  </Tabs>;
}
