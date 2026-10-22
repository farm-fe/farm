import type { ReactNode } from 'react';

/** Decorative language glyph; the visible menu label supplies its accessible name. */
export function LanguageIcon() {
  return <svg aria-hidden="true" focusable="false" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><text x="1" y="13" fill="currentColor" stroke="none" fontSize="13">文</text><path d="m13 21 4-11 4 11m-6-4h4" /></svg>;
}

export function NavigationIconLink({ href, label, children }: { href: string; label: string; children: ReactNode }) {
  return <a className="farm-docs-navigation-icon" href={href} aria-label={label} target="_blank" rel="noopener noreferrer">{children}</a>;
}

export function NavigationMenu({ label, name, children }: { label: ReactNode; name: string; children: ReactNode }) {
  return <details className="farm-docs-alternate farm-docs-navigation-menu" onKeyDown={event => {
    if (event.key !== 'Escape' || !event.currentTarget.open) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.open = false;
    event.currentTarget.querySelector('summary')?.focus();
  }}><summary>{label}</summary><nav aria-label={name}>{children}</nav></details>;
}
