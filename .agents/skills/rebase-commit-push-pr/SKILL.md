---
name: rebase-commit-push-pr
description: Fetch origin/main, rebase current branch, resolve conflicts safely, then commit and push changes; create a PR if none exists for the branch.
license: MIT
compatibility: Requires git repository with origin remote and GitHub CLI (gh) for PR checks/creation.
metadata:
  author: farm
  version: "1.0"
---

Safely sync a working branch with `origin/main`, handle rebase conflicts, then publish and open a PR if needed.

## Steps

1. Inspect branch and working tree
   - `git branch --show-current`
   - `git status --short`
   - If there are unresolved conflicts already, stop and ask for user direction.

2. Fetch and rebase onto main
   - `git fetch origin main`
   - `git rebase origin/main`

3. Resolve rebase conflicts (if any)
   - Check conflicted files: `git status --short`
   - Resolve each conflict intentionally (do not run destructive resets).
   - Stage resolved files: `git add <paths>` or `git rm <paths>`.
   - Continue: `git rebase --continue`.
   - Repeat until rebase finishes or explicit user input is required.

4. Commit user-requested local changes (only if needed)
   - If `git status --short` is empty, skip commit.
   - Else stage desired files (`git add -A` by default), then commit with a conventional message.

5. Push branch
   - `git push`
   - If push is rejected after rebase, use `git push --force-with-lease` only when user explicitly asked for rewritten-history push.

6. Ensure a PR exists
   - Detect current branch PR:
     - `gh pr list --head "$(git branch --show-current)" --json number,title,state,url`
   - If no PR exists, create one:
     - `gh pr create --fill`
     - If `--fill` fails due to missing metadata, provide explicit `--title` and `--body`.

7. Report result
   - Branch name
   - Rebase outcome (and conflict files resolved)
   - Commit hash/title created in this run (or note "no new commit")
   - Push destination
   - PR URL (existing or newly created)

## Guardrails

- Never use `git reset --hard`, `git checkout --`, or force push without explicit user approval.
- Do not amend commits unless explicitly requested.
- If hooks/tests fail during commit or push, surface the first failure and stop.
