import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
const root = process.env.FARM_DOCS_HEADER_ROOT || fileURLToPath(new URL('..', import.meta.url));
const theme = fs.readFileSync(path.join(root, 'website/src/docs/theme.tsx'), 'utf8');
const css = fs.readFileSync(path.join(root, 'website/src/css/farm-site.css'), 'utf8');
const navigation = theme.slice(theme.indexOf('function MainNavigation'), theme.indexOf('function RepositoryLink'));
test('primary navigation uses direct localized links on every page', () => {
 assert.match(navigation, /return <Link key={link.to}/);
 assert.match(navigation, /aria-current={page.kind === 'doc' && active === link.label/);
 assert.doesNotMatch(navigation, /const entries|const leaves|page.routePath === homeTarget/);
 assert.match(navigation, /NavigationMenu hover={false} label={community.label}/);
});
test('version and language selectors explicitly opt into the same hover component', () => {
 const alternates = theme.slice(theme.indexOf('function Alternates'), theme.indexOf('export function Navbar'));
 assert.match(alternates, /<NavigationMenu hover className="farm-site-alternate"/);
 assert.doesNotMatch(alternates, /<details/);
});
test('homepage and document header share the provider and scoped visual contract', () => {
 assert.match(theme, /SearchProvider.Provider value={WebsiteSearch}/);
 assert.doesNotMatch(theme, /value={home ? null : WebsiteSearch}/);
 assert.match(css, /.farm-site-shell > .farm-site-navbar {/);
});
