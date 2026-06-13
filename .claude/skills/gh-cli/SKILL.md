---
name: gh-cli
description: Use when working with GitHub via the gh CLI - managing PRs, issues, releases, repos, workflows, searching code, or calling the GitHub API. Triggers on tasks involving GitHub operations, PR creation/management, issue tracking, release workflows, or any gh command usage.
license: MIT
compatibility: Requires gh CLI installed and authenticated (gh auth login).
metadata:
  author: farm
  version: "1.0"
---

# GH CLI

Quick reference for the GitHub CLI (`gh`). Covers the most common operations in the farm repo workflow.

## Overview

`gh` is GitHub's official CLI tool. It wraps the GitHub API and Git operations into composable commands. Most commands accept `--json` for structured output and `--jq` for filtering.

## Prerequisites

```bash
gh auth status                          # verify authenticated
gh auth login                           # authenticate if needed
```

## Common Operations

### Pull Requests

| Operation | Command |
|-----------|---------|
| List PRs | `gh pr list --state open` |
| List PRs for current branch | `gh pr list --head "$(git branch --show-current)" --json number,title,state,url` |
| Create from current branch | `gh pr create --fill` |
| Create with explicit title/body | `gh pr create --title "feat: ..." --body "...description..."` |
| Create with body from file | `gh pr create --title "..." --body-file body.md` |
| View PR details | `gh pr view [number]` |
| View PR by branch | `gh pr view --head "$(git branch --show-current)"` |
| Check PR out locally | `gh pr checkout <number>` |
| View CI checks | `gh pr checks [number]` |
| Merge PR | `gh pr merge <number> --squash` |
| Close PR | `gh pr close <number>` |
| Comment on PR | `gh pr comment <number> --body "..."` |
| Request review | `gh pr edit <number> --add-assignee @me --add-reviewer @user` |
| Draft PR | `gh pr create --draft --title "..." --body "..."` |

### Issues

| Operation | Command |
|-----------|---------|
| List issues | `gh issue list --state open --limit 20` |
| Create issue | `gh issue create --title "..." --body "..."` |
| Create with type | `gh api repos/{owner}/{repo}/issues -X POST -f title=... -f body=... -f type="Bug"` |
| View issue | `gh issue view <number>` |
| Close issue | `gh issue close <number>` |
| Reopen issue | `gh issue reopen <number>` |
| Comment on issue | `gh issue comment <number> --body "..."` |
| Edit issue labels | `gh issue edit <number> --add-label "bug" --remove-label "wip"` |

**Note:** `gh issue create` does not support `--type`. Use `gh api` for issue types (see API section below).

### Repositories

| Operation | Command |
|-----------|---------|
| Clone a repo | `gh repo clone owner/repo` |
| Fork a repo | `gh repo fork owner/repo` |
| View repo info | `gh repo view --json nameWithOwner,description,defaultBranch` |
| List releases | `gh release list` |
| View release | `gh release view v1.0.0` |
| Create release | `gh release create v1.0.0 --title "..." --notes "..."` |

### Workflows & CI

| Operation | Command |
|-----------|---------|
| List workflows | `gh workflow list` |
| View workflow runs | `gh run list --workflow "CI" --limit 10` |
| Watch a run | `gh run watch <run-id>` |
| View run logs | `gh run view <run-id> --log` |
| Re-run failed | `gh run rerun <run-id> --failed` |
| Cancel a run | `gh run cancel <run-id>` |
| Download artifacts | `gh run download <run-id>` |

### Search

| Operation | Command |
|-----------|---------|
| Search issues | `gh search issues "query" --owner org --repo repo` |
| Search PRs | `gh search prs "query" --owner org --repo repo` |
| Search code | `gh search code "term" --owner org --repo repo` |

### API Access

Call arbitrary GitHub REST endpoints:

```bash
gh api repos/{owner}/{repo}/issues \
  -X POST \
  -f title="Title" \
  -f body="Body in markdown" \
  -f labels[]="bug" \
  --jq '{number, html_url}'
```

For GraphQL:

```bash
gh api graphql -f query='
  query($owner: String!, $repo: String!) {
    repository(owner: $owner, name: $repo) {
      issues(first: 5, states: OPEN) {
        nodes { number title }
      }
    }
  }
' -f owner=org -f repo=repo
```

**Key flags for `gh api`:**
- `-X METHOD` — HTTP method (GET, POST, PATCH, DELETE)
- `-f key=value` — form field (repeat for multiple values; use `-f key[]=val` for arrays)
- `-F key=@file` — file upload
- `--jq '...'` — filter JSON output with jq syntax
- `--paginate` — follow pagination automatically
- `-H "Header: value"` — custom headers

## Typical Patterns in Farm

### Create a PR for the current branch

```bash
git push -u origin "$(git branch --show-current)"
gh pr create --fill
```

If `--fill` fails, provide explicit title/body:

```bash
gh pr create --title "feat: description" --body "Closes #123"
```

### Check if a branch already has a PR

```bash
gh pr list --head "$(git branch --show-current)" --json number,title,state,url
```

Returns empty `[]` if no PR exists.

### View CI status for a PR

```bash
gh pr checks
# or for a specific PR
gh pr checks <number>
```

### Merge a PR with squash

```bash
gh pr merge --squash --delete-branch
```

## JSON Output and Filtering

Most `gh` commands support `--json` and `--jq`:

```bash
# Get PR number and URL
gh pr list --head "my-branch" --json number,url --jq '.[0] | {number, url}'

# Get specific field
gh pr view --json title --jq '.title'

# List PR numbers only
gh pr list --state open --json number --jq '.[].number'
```

## Tips

- Use `--json` + `--jq` instead of parsing text output
- `gh pr create --fill` uses branch commits for title/body — works best with conventional commits
- `gh api` is the escape hatch when built-in commands lack features (e.g., issue types)
- For GitHub Actions secrets: `gh secret set NAME --body "value" --repo owner/repo`
- `GH_TOKEN` env var or `GH_HOST` can target different GitHub instances
- Run `gh <command> --help` for full flag documentation on any command
