---
name: git-commit-push
description: Commit and push repository changes safely. Use when user asks to commit, push, or verify and push current changes.
license: MIT
compatibility: Requires git repository with remote configured.
metadata:
  author: farm
  version: "1.0"
---

Safely commit and push changes for the current branch.

## Steps

1. Check current branch and repo status
   - `git branch --show-current`
   - `git status --short`

2. Validate commit preconditions
   - If no changes: stop and report nothing to commit.
   - If merge conflicts exist: stop and ask user for conflict resolution direction.

3. Stage changes
   - Default: `git add -A`
   - If user specifies paths, only stage those.

4. Create commit
   - Use user-provided commit message.
   - If none is provided, draft a concise conventional message and ask for confirmation.
   - Run `git commit -m "<message>"`.

5. Push changes
   - Run `git push`.

6. Report result
   - Include branch name, commit hash, commit title, and push destination.
   - Include final `git status --short` result.

## Guardrails

- Never use destructive git operations (force push, reset --hard, checkout --) unless explicitly requested.
- Do not amend commits unless explicitly requested.
- If hooks fail, surface the first failure and stop.
