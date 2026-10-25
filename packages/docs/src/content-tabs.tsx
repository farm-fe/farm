import type { KeyboardEvent, ReactNode } from 'react';
import { Children, isValidElement, useEffect, useId, useState } from 'react';
import { useSite } from './client.js';

export interface TabItemProps { value: string; label: ReactNode; disabled?: boolean; children: ReactNode }
export interface TabsStorageMigration { key: string; values: readonly string[] }
export interface TabsProps { children: ReactNode; label?: string; defaultValue?: string; groupId?: string; storageMigration?: TabsStorageMigration }

export function TabItem({ children }: TabItemProps) { return <>{children}</>; }

/** Enhance existing React content; parsing and highlighting remain in Rust. */
export function Tabs({ children, label = 'Tabs', defaultValue, groupId, storageMigration }: TabsProps) {
  const site = useSite();
  const id = useId();
  const items = Children.toArray(children).filter(child => !(typeof child === 'string' && !child.trim())).map(child => {
    if (!isValidElement<TabItemProps>(child) || child.type !== TabItem) throw new Error('[DOCS_TABS] Tabs requires direct TabItem children');
    return child.props;
  });
  const values = new Set<string>();
  for (const item of items) {
    if (typeof item.value !== 'string' || !item.value.trim() || values.has(item.value)) throw new Error('[DOCS_TABS] values must be nonempty and unique');
    if (item.label === null || item.label === undefined || item.label === '') throw new Error('[DOCS_TABS] each item requires a label');
    values.add(item.value);
  }
  const available = items.filter(item => !item.disabled);
  const fallback = defaultValue ?? available[0]?.value;
  if (!available.some(item => item.value === fallback)) throw new Error('[DOCS_TABS] defaultValue must identify an enabled item');
  if (groupId !== undefined && (typeof groupId !== 'string' || !groupId.trim())) throw new Error('[DOCS_TABS] groupId must be nonempty');
  if (storageMigration && (!groupId || typeof storageMigration.key !== 'string' || !storageMigration.key.trim() || !Array.isArray(storageMigration.values) || storageMigration.values.some(value => typeof value !== 'string' || !value.trim()) || new Set(storageMigration.values).size !== storageMigration.values.length)) throw new Error('[DOCS_TABS] storageMigration requires a group, key and unique values');
  const [selected, setSelected] = useState(fallback);
  const [enhanced, setEnhanced] = useState(false);
  const active = available.some(item => item.value === selected) ? selected : fallback;
  const key = groupId === undefined ? null : `farm-docs:tabs:${JSON.stringify([site.basePath, groupId])}`;
  const signature = JSON.stringify(available.map(item => item.value));
  const migrationSignature = JSON.stringify(storageMigration);

  useEffect(() => {
    setEnhanced(true);
    if (!key) return;
    const apply = (value: unknown) => {
      if (typeof value === 'string' && available.some(item => item.value === value)) setSelected(value);
    };
    try {
      let stored = localStorage.getItem(key);
      if (stored === null && storageMigration) {
        const legacy = localStorage.getItem(storageMigration.key);
        if (legacy !== null && /^(0|[1-9]\d*)$/.test(legacy)) {
          const index = Number(legacy);
          const candidate = Number.isSafeInteger(index) ? storageMigration.values[index] : undefined;
          if (candidate !== undefined && available.some(item => item.value === candidate)) {
            stored = candidate;
            try { localStorage.setItem(key, candidate); } catch { /* Readable legacy state still applies in memory. */ }
          }
        }
      }
      apply(stored);
    } catch { /* Storage can be denied. */ }
    const storage = (event: StorageEvent) => {
      try { if (event.storageArea !== localStorage) return; } catch { return; }
      if (event.key !== key && event.key !== null) return;
      if (event.newValue === null) setSelected(fallback);
      else apply(event.newValue);
    };
    const sibling = (event: Event) => {
      const detail = (event as CustomEvent<{ key?: unknown; value?: unknown }>).detail;
      if (detail?.key === key) apply(detail.value);
    };
    window.addEventListener('storage', storage);
    window.addEventListener('farm-docs-tabs', sibling);
    return () => {
      window.removeEventListener('storage', storage);
      window.removeEventListener('farm-docs-tabs', sibling);
    };
  }, [key, signature, fallback, migrationSignature]);

  const choose = (value: string) => {
    setSelected(value);
    if (key) {
      try { localStorage.setItem(key, value); } catch { /* Interaction remains usable. */ }
      window.dispatchEvent(new CustomEvent('farm-docs-tabs', { detail: { key, value } }));
    }
  };
  const keyboard = (event: KeyboardEvent<HTMLAnchorElement>, value: string) => {
    const index = available.findIndex(item => item.value === value);
    let next: number;
    if (event.key === 'ArrowRight') next = (index + 1) % available.length;
    else if (event.key === 'ArrowLeft') next = (index + available.length - 1) % available.length;
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = available.length - 1;
    else if (event.key === ' ') { event.preventDefault(); choose(value); return; }
    else return;
    event.preventDefault();
    const target = available[next];
    choose(target.value);
    const targetIndex = items.indexOf(target);
    const element = document.getElementById(`${id}-tab-${targetIndex}`);
    element?.focus({ preventScroll: true });
    const list = element?.parentElement;
    if (element && list) {
      const itemBounds = element.getBoundingClientRect();
      const listBounds = list.getBoundingClientRect();
      if (itemBounds.left < listBounds.left) list.scrollLeft += itemBounds.left - listBounds.left;
      else if (itemBounds.right > listBounds.right) list.scrollLeft += itemBounds.right - listBounds.right;
    }
  };
  return <section className="farm-docs-tabs" aria-label={label}>
    <div className="farm-docs-tab-list" role={enhanced ? 'tablist' : undefined} aria-label={enhanced ? label : undefined} aria-orientation={enhanced ? 'horizontal' : undefined}>
      {items.map((item, index) => <a key={item.value} id={`${id}-tab-${index}`} href={`#${id}-panel-${index}`}
        role={enhanced ? 'tab' : undefined} aria-controls={enhanced ? `${id}-panel-${index}` : undefined}
        aria-selected={enhanced ? active === item.value : undefined} aria-disabled={item.disabled || undefined} data-active={active === item.value}
        tabIndex={enhanced ? (active === item.value && !item.disabled ? 0 : -1) : undefined}
        onClick={event => { if (!enhanced) return; event.preventDefault(); if (!item.disabled) choose(item.value); }}
        onKeyDown={event => { if (enhanced && !item.disabled) keyboard(event, item.value); }}>{item.label}</a>)}
    </div>
    {items.map((item, index) => <div key={item.value} id={`${id}-panel-${index}`} role={enhanced ? 'tabpanel' : 'region'}
      aria-labelledby={`${id}-tab-${index}`} data-active={active === item.value} hidden={enhanced && active !== item.value} tabIndex={enhanced ? 0 : undefined}>{item.children}</div>)}
  </section>;
}
