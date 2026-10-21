import { Children, isValidElement } from 'react';
import { CommandTabs as CompiledCommandTabs, type CommandTabsProps } from '@farmfe/docs/snippets-runtime';

export { CodeBlock } from '@farmfe/docs/snippets-runtime';
const order = ['npm', 'yarn', 'pnpm', 'bun'];
const storageMigration = { key: 'tabs.package.manager', values: order };

/** Website presentation of already compiled panels; no code parsing or rewriting. */
export function CommandTabs({ children }: CommandTabsProps) {
  const rank = (child: unknown) => isValidElement<{ 'data-command-tool'?: string }>(child)
    ? order.indexOf(child.props['data-command-tool'] ?? '') : -1;
  const panels = Children.toArray(children).sort((left, right) => rank(left) - rank(right));
  return <CompiledCommandTabs storageMigration={storageMigration}>{panels}</CompiledCommandTabs>;
}
