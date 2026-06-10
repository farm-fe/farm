import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';

const EXAMPLES_DIR = resolve(process.cwd(), 'examples');
const BROWSER_E2E_API_RE = /\b(startAndTest|watchAndTest)\s*\(|from\s+['"]playwright-chromium['"]/;

const failures = [];

for (const name of readdirSync(EXAMPLES_DIR).sort((a, b) => a.localeCompare(b))) {
  const examplePath = join(EXAMPLES_DIR, name);
  if (!statSync(examplePath).isDirectory()) continue;

  const specFile = join(examplePath, 'e2e.spec.mjs');
  if (!existsSync(specFile)) continue;

  const source = readFileSync(specFile, 'utf8');
  if (!BROWSER_E2E_API_RE.test(source)) {
    failures.push(
      `examples/${name}/e2e.spec.mjs must run browser e2e tasks via startAndTest/watchAndTest or Playwright. Move build-only or script-only checks out of e2e.spec.mjs.`
    );
  }
}

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}
