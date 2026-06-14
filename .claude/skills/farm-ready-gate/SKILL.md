---
name: farm-ready-gate
description: Run Farm verification with build-first constraints. For verify/acceptance, use npm run build only for core, rust plugins, and affected examples; do not run cargo check/clippy. Use when validating changes before merge or push.
license: MIT
compatibility: Requires Node, pnpm, cargo, and project dependencies.
metadata:
  author: farm
  version: "1.0"
---

Run repository verification with build-first constraints. For verify/acceptance, default to npm run build only.

## Verify Constraints (MANDATORY)

1. All build verification for core and Rust plugins must use `npm run build`.
2. Verify flow must not run `cargo build`, `cargo check`, or `cargo clippy`.
3. For examples, verify by running `npm run build` in each affected example directory.
4. Only run `pnpm run ready` when the user explicitly asks for full CI parity.

## Tiered Verification (choose one tier before running anything)

### Tier 1 — Bug Fixes & Simple/Localized Changes

Use when: fixing a bug, changing a single package/crate/plugin, adding a small localized feature.

**Steps (run in order, stop at first failure):**

1. **Build `@farmfe/core`** (always required — it is the central dependency):
  ```bash
  pnpm --filter @farmfe/core run build
  ```
2. **Build affected Rust plugin packages** (from each changed plugin directory):
  ```bash
   npm run build
  ```
3. **Build affected examples** (every example that exercises the changed feature):
  ```bash
  cd examples/<name> && npm run build
  ```

If all three steps pass, verification is PASS.

---

### Tier 2 — Complex / Multi-Package Changes & Pre-Release Gate

Use when: refactoring across multiple packages/plugins, changing plugin/compiler APIs, touching build infrastructure.

**Default verify flow (still build-only):**

```bash
pnpm --filter @farmfe/core run build
# then run npm run build in each changed rust-plugin/js-plugin/example package
```

**Optional CI parity flow (only if explicitly requested):**

```bash
pnpm run ready
```

Do not switch to this flow automatically.

## Ready Flow Reference (from scripts/ready.mjs)

When `pnpm run ready` is explicitly requested, the script runs checks in this order:

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

1. Determine tier based on change scope before running any commands.
2. For verify/acceptance, default to build-only checks using `npm run build`.
3. Do not run `cargo check` or `cargo clippy` in verify flow.
4. Run `pnpm run ready` only when explicitly requested by the user.
5. Do not claim success unless all chosen-tier steps exit with code 0.

## Output Format

- `Result`: PASS or FAIL
- `Command`: exact command executed
- `Failure Step`: if failed, map to ready flow item number/name
- `Next Action`: one concrete fix direction

## Guardrails

- Do not skip steps unless user explicitly requests a partial check.
- Preserve current git changes; do not revert files automatically.
- Keep logs summarized; include only the key failing lines.
