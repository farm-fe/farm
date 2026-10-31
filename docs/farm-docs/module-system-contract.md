# Module-system classification regression

The new-head Ubuntu unit job (119068225312) failed two plugin-script tests: actual `EsModule`, legacy expected `Hybrid`. Both legacy fixtures consume `require` dependencies alongside ESM declarations, but neither writes CommonJS exports.

The existing V2 contract in `merge_analyzed_module_systems` preserves ESM when static ESM declarations exist and no unresolved `module`/`exports` export usage is found. Dependency kinds alone do not determine the export system. Classifying these inputs as Hybrid lowers their static imports to require, changing conditional-export selection. A module with both ESM declarations and real CommonJS exports remains Hybrid; a locally bound `exports` variable must not be mistaken for the CommonJS global.

The test update leaves product code untouched. It retains the original require-consumer fixtures, asserts ESM and both metadata flags, adds real `module.exports` JavaScript and `exports` TypeScript fixtures that must remain Hybrid, and adds a shadowed local `exports` case that must remain ESM. Pure ESM and require-only CommonJS cases remain covered. No snapshots are updated.

Execution evidence is separate from classification assertions. Unmodified generated compiler fixtures preserve the ESM consumer's foo/bar/default exports and native fs.read identity; the real Hybrid fixture preserves its ESM name and default CommonJS age. A second controlled probe changes only the external node:fs specifier to a package with different import/require conditional exports, leaving generated import/require syntax intact: the ESM consumer selects the import branch, while real Hybrid lowering selects require and retains both export systems. This probe is diagnostic QA, not a claim that the original node:fs fixture itself has conditional exports.

Original remote and local red logs are retained in the local CI review directory. The finite plugin-script test target must pass after this test-only update; the currently running Docs/Vercel deployment has unchanged product inputs and remains independently monitored.

Windows finite verification passed both plugin-script tests across seven fixtures with snapshot update variables removed. The source contract probe also passed. The earlier red run failed both legacy assertions exactly as remote Ubuntu did.
