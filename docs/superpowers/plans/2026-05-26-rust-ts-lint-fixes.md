# Rust and TypeScript Lint Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the repository pass Rust formatting/clippy checks and TypeScript Biome/type-check diagnostics found in the current working tree.

**Architecture:** Keep fixes narrow and diagnostic-driven. Apply formatter output for Rust style-only failures, make the minimal Rust code change suggested by Clippy, and restore TypeScript package declaration resolution by ensuring `@farmfe/utils` exposes build scripts and type declarations for its exported entries.

**Tech Stack:** Rust nightly toolchain, Cargo fmt, Cargo clippy, pnpm 9.4.0, TypeScript, Biome 2.2.2.

---

## Current Diagnostics

- `pnpm exec biome check . --diagnostic-level=warn` passed with no fixes required.
- `cargo fmt --all --check` failed on formatting in:
  - `/home/runner/work/farm/farm/rust-plugins/tailwindcss/src/lib.rs`
  - `/home/runner/work/farm/farm/rust-plugins/tailwindcss/tests/mod.rs`
- `cargo clippy --workspace --all-targets -- -D warnings` failed in `/home/runner/work/farm/farm/crates/utils/src/lib.rs` because `path.split('?').last()` triggers `clippy::double_ended_iterator_last`.
- `pnpm -r --if-present type-check` initially failed because generated declarations were missing for packages that type-check against `dist`.
- After building `@farmfe/runtime`, type-check progressed to `@farmfe/js-plugin-electron`, which could not resolve `@farmfe/core` declarations.
- Building `@farmfe/core` failed because `@farmfe/utils/colors` has no generated `dist` declarations in the fresh checkout and `@farmfe/utils` has no `build` script or `types` metadata.

## File Responsibilities

- `/home/runner/work/farm/farm/docs/superpowers/plans/2026-05-26-rust-ts-lint-fixes.md`: tracks diagnostics and the repair sequence.
- `/home/runner/work/farm/farm/rust-plugins/tailwindcss/src/lib.rs`: Rust formatting-only fixes.
- `/home/runner/work/farm/farm/rust-plugins/tailwindcss/tests/mod.rs`: Rust formatting-only fixes.
- `/home/runner/work/farm/farm/crates/utils/src/lib.rs`: Clippy fix for efficient query suffix lookup.
- `/home/runner/work/farm/farm/packages/utils/package.json`: package metadata/build script fix so `@farmfe/utils` generates and exposes TypeScript declarations.

---

### Task 1: Apply Rust formatting

**Files:**
- Modify: `/home/runner/work/farm/farm/rust-plugins/tailwindcss/src/lib.rs`
- Modify: `/home/runner/work/farm/farm/rust-plugins/tailwindcss/tests/mod.rs`

- [ ] **Step 1: Run formatter**

Run: `cd /home/runner/work/farm/farm && cargo fmt --all`

Expected: command exits successfully and only formatting changes are made.

- [ ] **Step 2: Verify formatting**

Run: `cd /home/runner/work/farm/farm && cargo fmt --all --check`

Expected: command exits successfully with no diff output.

- [ ] **Step 3: Commit checkpoint**

Run through the agent progress tool with commit message: `style: format rust sources`

Expected: formatted Rust files are committed and pushed to the working PR branch.

### Task 2: Fix Clippy double-ended iterator warning

**Files:**
- Modify: `/home/runner/work/farm/farm/crates/utils/src/lib.rs:33-40`

- [ ] **Step 1: Confirm failing diagnostic**

Run: `cd /home/runner/work/farm/farm && cargo clippy -p farmfe_utils --all-targets -- -D warnings`

Expected: failure mentions `clippy::double_ended_iterator_last` at `crates/utils/src/lib.rs:39`.

- [ ] **Step 2: Apply minimal implementation**

Change `path.split('?').last().unwrap()` to `path.split('?').next_back().unwrap()` in `parse_query`.

- [ ] **Step 3: Verify package Clippy**

Run: `cd /home/runner/work/farm/farm && cargo clippy -p farmfe_utils --all-targets -- -D warnings`

Expected: command exits successfully.

- [ ] **Step 4: Commit checkpoint**

Run through the agent progress tool with commit message: `fix: satisfy farmfe_utils clippy`

Expected: Clippy fix is committed and pushed to the working PR branch.

### Task 3: Restore `@farmfe/utils` type declarations

**Files:**
- Modify: `/home/runner/work/farm/farm/packages/utils/package.json`

- [ ] **Step 1: Confirm declaration issue**

Run: `cd /home/runner/work/farm/farm && pnpm --filter @farmfe/core build`

Expected: failure reports missing exports from `@farmfe/utils/colors` or missing declarations for `@farmfe/utils/colors`.

- [ ] **Step 2: Add package build and types metadata**

Update `packages/utils/package.json` so it includes:
- `"types": "dist/index.d.ts"`
- root export `"types": "./dist/index.d.ts"`
- `./colors` export `"types": "./dist/color.d.ts"`
- `"scripts": { "build": "tsc -p tsconfig.json", "type-check": "tsc -p tsconfig.json --noEmit" }`

- [ ] **Step 3: Verify utils build**

Run: `cd /home/runner/work/farm/farm && pnpm --filter @farmfe/utils build`

Expected: command exits successfully and creates generated files under `packages/utils/dist`.

- [ ] **Step 4: Verify core build/type declarations**

Run: `cd /home/runner/work/farm/farm && pnpm --filter @farmfe/core build`

Expected: TypeScript can resolve `@farmfe/utils/colors` declarations and core build proceeds past previous missing-export errors.

- [ ] **Step 5: Commit checkpoint**

Run through the agent progress tool with commit message: `fix: expose utils type declarations`

Expected: package metadata fix is committed and pushed to the working PR branch.

### Task 4: Run requested repository diagnostics

**Files:**
- No direct edits expected.

- [ ] **Step 1: Run Biome**

Run: `cd /home/runner/work/farm/farm && pnpm exec biome check . --diagnostic-level=warn`

Expected: command exits successfully.

- [ ] **Step 2: Run Rust format check**

Run: `cd /home/runner/work/farm/farm && cargo fmt --all --check`

Expected: command exits successfully.

- [ ] **Step 3: Run Rust Clippy**

Run: `cd /home/runner/work/farm/farm && cargo clippy --workspace --all-targets -- -D warnings`

Expected: command exits successfully, or any remaining unrelated diagnostics are recorded with exact file paths before fixing in a follow-up task.

- [ ] **Step 4: Run TypeScript type checks**

Run: `cd /home/runner/work/farm/farm && pnpm -r --if-present type-check`

Expected: command exits successfully, or any remaining unrelated diagnostics are recorded with exact file paths before fixing in a follow-up task.

- [ ] **Step 5: Commit final checkpoint**

Run through the agent progress tool with commit message: `chore: verify rust and ts checks`

Expected: all requested checks and any additional minimal fixes are committed and pushed.
