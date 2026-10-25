import { type CommandTabsProps, CommandTabs as CompiledCommandTabs } from '@farmfe/docs/snippets-runtime';
import { Children, isValidElement } from 'react';

export { CodeBlock } from '@farmfe/docs/snippets-runtime';

const order = ['npm', 'yarn', 'pnpm', 'bun'] as const;
const labels = Object.fromEntries(order.map(tool => [tool, <span className="farm-docs-tab-label"><img src={`/img/package-managers/${tool}.svg`} alt="" aria-hidden="true" />{tool}</span>]));
const storageMigration = { key: 'tabs.package.manager', values: order };

/** Website presentation of already compiled panels; no code parsing or rewriting. */
export function CommandTabs({ children }: CommandTabsProps) {
  const rank = (child: unknown) => isValidElement<{ 'data-command-tool'?: string }>(child)
    ? (order as readonly string[]).indexOf(child.props['data-command-tool'] ?? '') : -1;
  const panels = Children.toArray(children).sort((left, right) => rank(left) - rank(right));
  return <CompiledCommandTabs storageMigration={storageMigration} labels={labels}>{panels}</CompiledCommandTabs>;
}
