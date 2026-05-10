# Example Testing Guide (`scripts/test-examples.mjs`)

## Scope

This guide explains how to validate one example or a subset of examples using
`scripts/test-examples.mjs`, including the argument mapping and expected output.

## Prerequisites

- Run from repository root.
- Dependencies are installed (`pnpm install`).
- Use `--skip-build-js-plugins` when iterating on non-JS-plugin changes to
  reduce validation time.

## Quick Start

Build only one example:

```bash
node scripts/test-examples.mjs --skip-build-js-plugins --example rust-plugin-wasm
```

Build from one example to the end of the sorted examples list:

```bash
node scripts/test-examples.mjs --skip-build-js-plugins --from rust-plugin-wasm
```

## Argument Mapping

| Argument | Meaning | Notes |
| --- | --- | --- |
| `--example <name>` | Build only one example. | Supports `--example=name` and `FARM_EXAMPLE`. |
| `--from <name>` | Build from the given example to the end. | Alias: `--start-from`. Supports `--from=name` / `--start-from=name` and `FARM_EXAMPLE_START_FROM`. |
| `--skip-build-js-plugins` | Skip JS plugin build stage. | Alias: `--skip-build-js-plugin`; also supports `--skip-build-js-plugins=true` and `FARM_SKIP_BUILD_JS_PLUGINS=1/true`. |

Precedence in example selection:

1. `--example` (single example mode)
2. `--from` / `--start-from` (range mode)
3. Default: build all examples

## Expected Output Signals

For single-example mode:

- `Building only example: <name>`
- `Building 1 examples...`
- `Building examples\\<name>`

If the example does not exist:

- `Example '<name>' was not found under ./examples`

## Troubleshooting

- If you see another example (for example `examples\\arcgis`) while using
  `--example`, verify that:
  - You are running the command from repo root.
  - The command uses `--example`, not only `--from`.
  - `<name>` exactly matches a directory under `examples/`.
- If command exits with non-zero, inspect the first failed example build log to
  determine whether it is an example issue or orchestration issue.
