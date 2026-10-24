// Test tooling only. Keep later phase reruns from replacing historical checkpoint artifacts.
import fs from 'node:fs/promises';
import path from 'node:path';
export async function evidenceDirectory(repository, defaultPhase = 'p03') {
  const phase = process.env.CODEX_DOCS_EVIDENCE_PHASE ?? defaultPhase;
  if (!/^p\d{2}$/.test(phase)) throw new Error(`Invalid evidence phase: ${phase}`);
  const attempt = process.env.CODEX_DOCS_EVIDENCE_ATTEMPT;
  if (attempt !== undefined && !/^[a-z0-9][a-z0-9-]{0,95}$/.test(attempt)) throw new Error(`Invalid evidence attempt: ${attempt}`);
  const directory = path.join(repository, 'docs/farm-docs/evidence', phase, ...(attempt ? [attempt] : []));
  await fs.mkdir(directory, { recursive: true });
  return directory;
}
