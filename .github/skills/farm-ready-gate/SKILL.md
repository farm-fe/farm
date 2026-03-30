---
name: farm-ready-gate
description: Run Farm's full ready gate and confirm all tests/checks pass. Use when user asks to verify everything before merge, release, or push; keywords: ready, full check, all tests, CI parity, npm run ready.
license: MIT
compatibility: Requires Node, pnpm, cargo, and project dependencies.
metadata:
  author: farm
  version: "1.0"
---

Run the repository readiness gate and verify all quality checks pass.

## Primary Command

Use the same top-level workflow as the repository ready script:

```bash
pnpm run ready
```

This maps to `node scripts/ready.mjs` and should be the default path.

## Ready Flow Reference (from scripts/ready.mjs)

The ready script runs checks in this order:

1. Install dependencies (`pnpm install`)
2. Clean artifacts (`node ./scripts/clean.mjs`)
3. Spell check (`npx cspell "**" --gitignore`)
4. Build core/plugins/cli via `runTaskQueue`
5. Cargo check (`cargo check --color always --all --all-targets`)
6. Cargo clippy (`cargo clippy`)
7. TypeScript checks (`pnpm run --filter "@farmfe/*" type-check`)
8. Unit tests (`pnpm run test`)
9. Rust tests (`cargo test -j <cpu_based_jobs>`)
10. Build core CJS (`buildCoreCjs`)
11. Build examples (`buildExamples`)
12. E2E tests (`pnpm run test-e2e`)

## Execution Policy

1. Prefer running `pnpm run ready` once first.
2. If it fails, report the exact failing step and error excerpt.
3. Optionally rerun only the failed step for a clearer diagnosis.
4. Do not claim success unless the overall ready command exits with code 0.

## Output Format

- `Result`: PASS or FAIL
- `Command`: exact command executed
- `Failure Step`: if failed, map to ready flow item number/name
- `Next Action`: one concrete fix direction

## Guardrails

- Do not skip steps unless user explicitly requests a partial check.
- Preserve current git changes; do not revert files automatically.
- Keep logs summarized; include only the key failing lines.
