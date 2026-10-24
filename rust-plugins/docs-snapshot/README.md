# @farmfe/plugin-docs-snapshot

Native Rust snapshot capture and verification for the Farm Docs workflow. The default npm export returns a Rust plugin descriptor; JavaScript only selects the binary and passes serialized options.

`SnapshotOptions` has two modes. `capture` receives `planPath`, `receiptPath`, `seedPath`, a browser/server `target` and `outputPath`. `verify` receives `planPath` and a `verification` object containing browser/server receipts and hashes, write inventory, public/server directories and template path. See the shipped declaration for exact field names.

These paths and receipts belong to one workflow generation. Let `@farmfe/docs` orchestrate them; do not reuse another build's receipts or expose private server plans as public site resources. This plugin alone does not render a website.

Use a matching Core/native distribution. Final tarball, platform and isolated consumer verification remains in progress. Farm LICENSE and the embedded resource notices in THIRD_PARTY_NOTICES.md and licenses must accompany distribution.
