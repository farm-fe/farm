# Actual suite receipts and candidate ledger

> Historical staged implementation notes below. Current live entrypoints are
> documented in `independent-incremental-chains.md` and `ci-suite-selection.md`.
> Lightweight local contracts have now run. Generic shadow output remains
> informational; it is not the authority for live byte or suite decisions.


The third batch changes real platform steps, not only plan output. The existing
TypeScript npm test command keeps its configured options and gains Vitest's JSON
reporter. `ci-suite-execution.mjs` requires the discovered committed spec files
to match the runner's actual files, with nonempty passing assertion results and
matching case totals. `--passWithNoTests` cannot provide valid evidence.

The browser orchestrator keeps its existing build/browser behavior and records
the real terminal worker results when the CI receipt variable is set. A zero
runnable count, missing terminal example, worker failure or skipped selected
case cannot establish suite coverage. Existing non-runnable Examples remain
explicit discovery exclusions, not positive execution evidence.

Each primary platform uploads compact TS and browser receipts with the actual
commit/tree, run/attempt, ABI, discovered IDs, executed IDs and per-file case
counts. No test body, arbitrary assertion message, environment dump or raw
Vitest report is uploaded as a receipt. Original process execution retains the
bounded command helper; no local process or test has been started.

The existing performance summary uses its current read-only permissions to
download the exact six current-run suite artifacts, validates their namespace,
source identity and nonempty counts, and uploads a candidate execution ledger.
The final shadow outcome now occurs after this upload so later failures are
not recorded as an earlier successful job outcome.

This ledger has a different schema and namespace from build-byte baselines. It
is explicitly a candidate requiring independently completed GitHub producer/run
and acceptance checks. `ci-execution-evidence.py` verifies the fixed producer and
upload step, re-downloads original suite receipt ZIPs, verifies their digests and
matches each with the actual successful platform job/upload step. It does not
accept a ledger's self-reported positive count as an execution proof.

## Limits before omission can be enabled

The initial proposed leaf scope is private engineering Markdown under
`docs/experiences`, with ordinary Git file mode. Every other tracked byte and
mode remains part of its acceptance input fingerprint. This policy is encoded
only in candidate input calculation; no workflow condition currently omits any
task. Deleted, new or modified executable-mode documentation is not silently
treated as unchanged product input.

The candidate currently covers actual TypeScript and browser counts. Independent
Examples build/create-farm counts and Unit acceptance need their own evidence;
they are not inferred from browser cases. Unit continues to run normally.
Cross-run lookup/selection and a final omission gate for this execution-ledger
schema have not yet been connected. No CI native byte resolver is enabled.

The separate native byte-baseline path still rejects incomplete toolchain/linker
measurement. A real producer wrapper and independently bound measured context
reader remain necessary before same-chain binary reuse. CI and release bytes
remain completely separate. Incremental release execution is still unfinished.

The source changes, contract sources and workflow steps in this batch have not
been tested locally or pushed. They require independent review and remote CI;
no speedup, successful ledger production or usable end-to-end release is claimed.
