---
name: no-skip-git-hooks
description: Rule forbidding --no-verify and --no-gpg-sign when committing
metadata:
  type: feedback
---

Always run git pre-commit hooks (lint-staged, biome, husky, etc.) when committing. Never use `--no-verify` or `--no-gpg-sign` flags with `git commit`.

**Why:** Skipping hooks hides real CI failures. If a hook fails, fix the issue and re-commit properly. The hooks catch biome lint/format errors, type errors, and other CI-blocking problems before they reach CI.

**How to apply:** Never pass `--no-verify` to `git commit`. If hooks fail, fix the reported issues and commit again normally.
