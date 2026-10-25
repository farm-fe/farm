import type { ReactNode } from 'react';

/** Decorative language glyph; the visible menu label supplies its accessible name. */
export function LanguageIcon() {
  return <svg aria-hidden="true" focusable="false" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><text x="1" y="13" fill="currentColor" stroke="none" fontSize="13">文</text><path d="m13 21 4-11 4 11m-6-4h4" /></svg>;
}

export function NavigationIconLink({ href, label, children }: { href: string; label: string; children: ReactNode }) {
  return <a className="farm-docs-navigation-icon" href={href} aria-label={label} target="_blank" rel="noopener noreferrer">{children}</a>;
}

export function NavigationMenu({ label, name, children, className = '', hover = false }: { label: ReactNode; name: string; children: ReactNode; className?: string; hover?: boolean }) {
  return <details className={`farm-docs-alternate farm-docs-navigation-menu ${className}`}
    onPointerEnter={event => { if (hover && event.pointerType === 'mouse') event.currentTarget.open = true; }}
    onPointerLeave={event => {
      if (!hover || event.pointerType !== 'mouse' || event.currentTarget.contains(event.relatedTarget as Node | null)) return;
      if (!event.currentTarget.contains(document.activeElement)) event.currentTarget.open = false;
    }}
    onBlur={event => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) event.currentTarget.open = false; }}
    onClick={event => { if ((event.target as Element).closest('a')) event.currentTarget.open = false; }}
    onKeyDown={event => {
      const menu = event.currentTarget;
      const links = Array.from(menu.querySelectorAll<HTMLAnchorElement>('nav a[href]'));
      const index = links.indexOf(document.activeElement as HTMLAnchorElement);
      if (event.key === 'Escape' && menu.open) { event.preventDefault(); event.stopPropagation(); menu.open = false; menu.querySelector('summary')?.focus(); }
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        event.preventDefault(); menu.open = true;
        const next = index < 0 ? (event.key === 'ArrowDown' ? 0 : links.length - 1) : (index + (event.key === 'ArrowDown' ? 1 : -1) + links.length) % links.length;
        links[next]?.focus();
      }
    }}><summary>{label}</summary><nav aria-label={name}>{children}</nav></details>;
}
