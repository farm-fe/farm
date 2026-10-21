import type {ThemeProps} from '@farmfe/docs/theme';
export function Footer({site}:ThemeProps) {
  return <footer data-testid="orchard-footer">{site.title} · Orchard cooperative</footer>;
}
