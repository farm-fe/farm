import type { ComponentType, HTMLAttributes } from 'react';

export {
  TabItem,
  type TabItemProps,
  Tabs,
  type TabsProps,
  type TabsStorageMigration
} from './content-tabs.js';
export type MDXComponents = Record<
  string,
  | ComponentType<HTMLAttributes<HTMLElement>>
  | ComponentType<Record<string, unknown>>
>;
