import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';

// Static guard, not browser or full React lifecycle acceptance. Provider hooks must
// belong to the provider's React fiber when switching between website/home routes.
test('search provider is a component slot rather than a directly invoked function', () => {
  const source = fs.readFileSync(new URL('../packages/docs-theme-default/src/search.tsx', import.meta.url), 'utf8');
  assert.match(source, /createContext<ComponentType<SearchProps>\s*\|\s*null>/);
  assert.match(source, /const Provider = useContext\(SearchProvider\)/);
  assert.match(source, /if \(Provider\) return <Provider site=\{site\} \/>/);
  assert.doesNotMatch(source, /(?:provider|Provider)\(\{\s*site\s*\}\)/);
});
