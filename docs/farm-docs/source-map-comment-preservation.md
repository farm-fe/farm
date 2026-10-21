# Source-map composition and preserved comments

The CI compiler suite exposed a genuine license-comment regression alongside older output snapshots. AST spans were already relocated by each original source-file interval, but comments still used the earlier direct-offset calculation. Their attachment positions no longer matched the emitted statements.

The runtime renderer and concatenate pipeline now merge source maps and comments together, using the same relocation intervals for AST spans, comment attachments and comment spans. This also handles a second composition of an already hoisted multi-file source map. Existing public merge function signatures remain available.

The original script fixture requiring its `@license` comment was retained: it failed before the fix and passes afterward without updating its expected output. Added nested-merge coverage verifies both attachment and original file identity. At the Windows checkpoint, compiler tests passed 125 with one ignored, toolkit library tests passed 35, and runtime/HMR tests passed 24; snapshot update mode was disabled for regression verification.

Remaining fixture changes were separately reviewed. Twelve of 21 changed JavaScript fixtures have identical normalized ASTs and only string quote formatting differs. The others reflect already implemented external namespace getters, raw Node CommonJS require behavior, local createRequire binding semantics, hybrid ESM/CJS output and the existing resource transaction runtime. Execution probes checked the source-level export contracts. Getter descriptors and key order changes are explicit, not described as universal semantic equivalence. No fixture was accepted by deleting its license-preservation expectation.

Native product and real website acceptance follows a normal Core build and fresh preview checks; these tests alone are not a clean Linux deployment result.
