import type { PublicSidebarItem } from '@farmfe/docs/theme';
export interface SidebarSection {
  id: string;
  prefixes: readonly string[];
}
/** Most specific configured document prefix wins; no website names are built in. */
export function sidebarSection(
  id: string,
  sections: readonly SidebarSection[],
  fallback: string
): string {
  let selected = fallback;
  let length = -1;
  for (const section of sections)
    for (const prefix of section.prefixes) {
      if (
        (id === prefix || id.startsWith(prefix + '/')) &&
        prefix.length > length
      ) {
        selected = section.id;
        length = prefix.length;
      }
    }
  return selected;
}
export function filterSidebarSection(
  items: PublicSidebarItem[],
  sections: readonly SidebarSection[],
  active: string,
  fallback: string
): PublicSidebarItem[] {
  const result: PublicSidebarItem[] = [];
  for (const item of items) {
    if (item.type === 'category') {
      const children = filterSidebarSection(
        item.items,
        sections,
        active,
        fallback
      );
      if (children.length) result.push({ ...item, items: children });
    } else if (
      item.type === 'doc' &&
      sidebarSection(item.id, sections, fallback) === active
    )
      result.push(item);
  }
  // Keep auxiliary external links only alongside a retained group, in their original position.
  if (result.length) {
    let position = 0;
    for (const item of items) {
      if (item.type !== 'doc' && item.type !== 'category')
        result.splice(position++, 0, item);
      else if (
        item.type === 'doc'
          ? result.includes(item)
          : result.some(
              (candidate) =>
                candidate.type === 'category' && candidate.label === item.label
            )
      )
        position++;
    }
  }
  return result;
}
