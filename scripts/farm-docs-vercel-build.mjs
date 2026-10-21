// Compatibility entry for existing project commands. Online source deployment
// is retired; do not restore Cargo/rustup or workspace build fallback here.
console.error('[FARM_DOCS_SOURCE_DEPLOYMENT_RETIRED] Online Farm source builds are disabled. Deployment requires an independent consumer of an approved, published v2 release set via scripts/farm-docs-registry-consumer.mjs. Required formal packages are not yet published; no source fallback is allowed.');
process.exitCode = 1;
