# Changeset Scenarios & Examples

## Bug Fix

```markdown
---
"@saleor/configurator": patch
---

Fix category parent reference not being set during deployment

Categories with parent slugs now correctly link to their parent
categories during the deploy operation.
```

## New Feature

```markdown
---
"@saleor/configurator": minor
---

Add diff command for previewing configuration changes

New `diff` command shows what would change before deploying:
- Displays creates, updates, and deletes
- Color-coded output for easy reading
- Supports `--json` flag for programmatic use
```

## Breaking Change

```markdown
---
"@saleor/configurator": major
---

Change attribute configuration format

**BREAKING**: Attribute values are now defined inline instead of by reference.

Before:
```yaml
attributes:
  - name: Color
    values:
      - red
      - blue
```

After:
```yaml
attributes:
  - name: Color
    values:
      - name: Red
        slug: red
      - name: Blue
        slug: blue
```

Migration: Update your config.yml to use the new format.
```

## Multiple Related Changes

```markdown
---
"@saleor/configurator": minor
---

Improve deployment reliability and progress reporting

- Add retry logic for failed GraphQL operations
- Display progress bar during bulk deployments
- Report partial failures at the end of deployment
- Add `--continue-on-error` flag to proceed despite failures
```

## Reference Attributes Support

```markdown
---
"@saleor/configurator": minor
---

Add support for reference attributes with entityType field

- Attributes of type REFERENCE now require an entityType field
- Introspection properly captures entity type references
- Deploy correctly handles reference attribute creation
```

## Consolidated Changeset Example

When multiple changesets should be combined before release:

```markdown
---
"@saleor/configurator": minor
---

Multiple improvements to bulk operations

This release includes several related improvements:

- Add retry logic for failed GraphQL operations (#123)
- Display progress bar during bulk deployments (#124)
- Report partial failures at the end of deployment (#125)
- Add `--continue-on-error` flag (#126)
```

## Writing Good Descriptions

### Do's

```markdown
---
"@saleor/configurator": minor
---

Add bulk product import command

New `import` command allows importing products from CSV files:
- Supports mapping CSV columns to product fields
- Validates data before import
- Reports import progress and errors
```

### Don'ts

```markdown
---
"@saleor/configurator": patch
---

fix bug
```

### Best Practices

1. **Start with a verb**: Add, Fix, Update, Remove, Improve
2. **Be specific**: What changed and why
3. **Include context**: Reference issue numbers if applicable
4. **Document migration**: For breaking changes

## Analyzing Changes for Bump Type

### Check git diff

```bash
# See what changed since last release
git log --oneline main..HEAD

# See detailed changes
git diff main..HEAD -- src/
```

### Key Questions

1. **Did the public API change?**
   - CLI commands modified → minor/major
   - Configuration schema changed → patch/minor/major
   - New features added → minor

2. **Could this break existing users?**
   - Yes, requires code changes → major
   - Yes, but has fallback → minor
   - No → patch

3. **Is this user-facing?**
   - Yes, new capability → minor
   - Yes, improved existing → patch
   - No, internal only → patch

## Consolidating Changesets

```bash
# View pending changesets
ls .changeset/

# Manually consolidate by:
# 1. Reading all changeset files
# 2. Creating a single comprehensive changeset
# 3. Deleting the individual files
```

## Pre-Release Versions

For beta/alpha releases:

```bash
# Enter pre-release mode
npx changeset pre enter beta

# Create changesets as normal
pnpm changeset

# Exit pre-release mode
npx changeset pre exit
```
