---
description: Commit staged or all workspace changes and push the current branch safely with clear verification output
---

Run a safe git commit and push workflow for the current repository.

**Input**: Optional commit message. If omitted, infer from changed files and ask for confirmation before committing.

## Steps

1. Check branch and repo state
   - Run `git branch --show-current`
   - Run `git status --short`

2. Verify there are changes to commit
   - If clean: report "nothing to commit" and stop.

3. Choose what to commit
   - Default: commit all tracked/untracked changes (`git add -A`)
   - If user asked for partial commit: stage only requested paths.

4. Prepare commit message
   - If provided in input: use it.
   - Else: draft a concise conventional message from the diff and ask for confirmation.

5. Commit
   - Run `git commit -m "<message>"`
   - If commit fails due hooks/lint/tests, report the first failing step and stop.

6. Push
   - Run `git push` to current branch upstream.

7. Report outcome
   - Show branch, commit hash, commit title, and push target.
   - Show final `git status --short` to confirm clean tree.

## Guardrails

- Never run destructive commands like reset/rebase/force-push unless user explicitly asks.
- Do not amend commits unless explicitly requested.
- If there are merge conflicts, stop and ask user how to proceed.
- Keep output short and actionable.
