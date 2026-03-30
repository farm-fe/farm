# Farm — AI Agent Instructions

This file provides project-level context for AI coding assistants working in
this repository.

## Project Overview

**Farm** is an extremely fast, Vite-compatible web build tool written in Rust.
It is a monorepo that contains the core compiler (Rust), official JS/Rust
plugins, runtime packages, a CLI, a documentation website, and examples.

- Website: <https://farmfe.org>
- npm: `@farmfe/core`
- License: MIT

---

## Repository Structure

```
farm/
├── crates/            Rust workspace — compiler core and most Rust plugins
│   ├── compiler/      Main compilation engine
│   ├── core/          Rust/Node bridge (napi-rs)
│   ├── plugin_*/      Official Rust plugins (CSS, HTML, JSON, resolve …)
│   └── node/          Node.js native bindings
├── packages/          TypeScript packages (runtime, CLI, etc.)
├── js-plugins/        Official JavaScript plugins (postcss, less, sass, svgr, dts, visualizer, tailwindcss, electron)
├── rust-plugins/      Community-maintained Rust plugins
├── examples/          Standalone example projects (each is its own workspace package)
├── website/           Docusaurus documentation site
├── scripts/           Build, release and tooling scripts
└── e2e/               Playwright E2E tests
```

---

## Key Commands

| Command | Purpose |
|---------|---------|
| `pnpm bootstrap` | First-time setup: install deps + build core packages |
| `pnpm run ready` | Full CI gate — install, clean, build, all tests (see below) |
| `pnpm run test` | Unit tests (vitest) |
| `cargo test` | Rust unit tests |
| `pnpm run test-e2e` | Playwright E2E tests |
| `pnpm run check` | Biome lint + format |
| `cargo clippy` | Rust linter |
| `pnpm run spell-check` | cspell across all files |

### Ready Gate (full CI parity)

`pnpm run ready` runs in order:

1. `pnpm install`
2. `node ./scripts/clean.mjs`
3. `npx cspell "**" --gitignore`
4. Build core / JS & Rust plugins / CLI (`runTaskQueue`)
5. `cargo check --all --all-targets`
6. `cargo clippy`
7. `pnpm run --filter "@farmfe/*" type-check`
8. `pnpm run test`
9. `cargo test -j <N>`
10. Build core CJS
11. Build examples
12. `pnpm run test-e2e`

---

## Tech Stack & Conventions

### Rust
- Rust edition 2021, toolchain pinned in `rust-toolchain.toml`
- Formatting via `rustfmt.toml`; lint with `cargo clippy`
- Crates share workspace dependencies declared in the root `Cargo.toml`

### TypeScript / JavaScript
- **Formatter + linter:** Biome (`biome.json`). Run `pnpm run check` before committing.
- **Package manager:** pnpm v9+ with workspaces (`pnpm-workspace.yaml`)
- **Module format:** ESM in source; dual CJS+ESM output for published packages
- **Type checking:** `tsc` per-package via `type-check` script
- All TS config files extend `tsconfig.base.json`

### Git
- Commit messages follow Conventional Commits (`feat:`, `fix:`, `chore:`, etc.)
- Changesets managed via `@changesets/cli` — run `npx changeset` when bumping a package version
- PR titles are linted by `lint-pr-title.yml` — must match Conventional Commits

---

## Coding Guidelines

- **Minimal changes.** Only touch files directly required by the task.
- **No new features** unless explicitly requested.
- **Security.** No use of `eval`, dynamic `require` with user input, or unsafe Rust `unwrap()` on user-facing paths.
- **Tests.** Add or update tests when fixing bugs or adding features.
- **Docs.** Update `website/docs/` when changing public API, config options, or plugin interfaces.

---

## Available AI Agents

| Agent | Trigger | File |
|-------|---------|------|
| **FarmFE Docs Sync** | After code changes that affect docs; finding doc/code discrepancies | `.github/agents/farmfe-docs-sync.agent.md` |
| **Explore** | Read-only codebase Q&A, researching architectures, tracing code paths | Built-in subagent |

---

## Available Skills (Slash Commands)

Skills are on-demand workflows invoked as slash commands in VS Code Copilot chat. They are discovered automatically from two directories:

| Directory | Scope | How to add a skill |
|-----------|-------|--------------------|
| `.github/skills/<name>/SKILL.md` | Workspace-shared (checked in, affects whole team) | Create the folder + `SKILL.md` with YAML frontmatter |
| `.agents/skills/<name>/SKILL.md` | Workspace-local (checked in, agent-only utilities) | Same structure; place in `.agents/skills/` |

Type `/` in Copilot Chat to browse all available slash commands.

### Project Skills (`.github/skills/`)

| Skill | Slash Command | Description |
|-------|---------------|-------------|
| `git-commit-push` | `/git-commit-push` | Safe commit + push workflow with guardrails |
| `farm-ready-gate` | `/farm-ready-gate` | Run the full CI readiness gate (`pnpm run ready`) |
| `rebase-commit-push-pr` | `/rebase-commit-push-pr` | Fetch/rebase onto `origin/main`, resolve conflicts, push, and ensure a PR exists |
| `openspec-propose` | `/opsx:propose` | Propose a new change with full artifacts |
| `openspec-apply-change` | `/opsx:apply` | Implement tasks from an OpenSpec change |
| `openspec-explore` | `/opsx:explore` | Thinking-partner explore mode |
| `openspec-archive-change` | `/opsx:archive` | Archive a completed change |
| `a5c-ai-docusaurus` | Built-in | Deep Docusaurus integration for the website |

### Agent Utility Skills (`.agents/skills/`)

These skills ship with the agent toolchain and are available in all workspaces that use the same agent setup.

| Skill | Trigger phrases | Description |
|-------|----------------|-------------|
| `rust-engineer` | Rust, Cargo, ownership, lifetimes, async | Writes, reviews, and debugs idiomatic Rust. Use for any Rust implementation work in `crates/` or `rust-plugins/`. |
| `rust-best-practices` | Rust review, idiomatic Rust, borrowing, cloning | Apollo-based Rust best practices guide. Use when reviewing or refactoring existing Rust code. |
| `agent-browser` | open website, fill form, screenshot, scrape, browser automation | Browser automation CLI for AI agents. Useful for testing Farm's dev server output in a real browser. |
| `find-skills` | find a skill, how do I do X, is there a skill | Helps discover and install additional agent skills. |
| `self-improving-agent` | (auto-triggers on skill completion/error) | Multi-memory self-correction agent that learns from skill outcomes and continuously improves the codebase. |

### How to Create a New Skill

1. Pick a scope: shared team workflow → `.github/skills/`; agent utility → `.agents/skills/`.
2. Create the directory: `mkdir .github/skills/<your-skill-name>`.
3. Create `SKILL.md` with required frontmatter:

```yaml
---
name: your-skill-name        # must match folder name
description: "Use when ..." # discovery surface — be specific
license: MIT
---

## Steps
...
```

4. Commit the file. The slash command `/your-skill-name` will appear in Copilot Chat immediately.

---

## Plugin Architecture

- **Rust plugins** live in `crates/plugin_*` and are compiled into the core binary.
- **JS plugins** live in `js-plugins/` and are published as individual npm packages under `@farmfe/js-plugin-*`.
- Every plugin must implement the `Plugin` trait (Rust) or the `JsPlugin` interface (TS).
- Vite plugins work in Farm via the vite-adapter compatibility layer; no wrapper needed.

---

## Do Not

- Run `git push --force` or `git reset --hard` without explicit user approval.
- Delete files/branches or drop database content without confirmation.
- Modify `pnpm-lock.yaml` manually — always regenerate with `pnpm install`.
- Edit generated files (`.d.ts` in `dist/`, `typed-router.d.ts`, `auto-imports.d.ts`).
- Bypass lint/format hooks with `--no-verify`.
