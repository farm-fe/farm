# V2 glob replacement

Before the first stable V2 cohort, Farm removes Wax and its old public re-export rather than retaining a second glob implementation. Use `farmfe_core::glob`; `is_match` is fallible and walk errors must be handled. Optional Wax diagnostic constructors/traits are intentionally removed. Fixture discovery explicitly remains exhaustive, while the import transform and HMR retain their separate Vite reference profiles.

The three obsolete nested/template Cargo.lock files are removed: workspace members use the authoritative workspace lock, and generated Rust plugins resolve their exact approved Core version when the corresponding registry release is available. They must not ship a stale lock that restores an old Core/Wax graph.

`@farmfe/js-plugin-vuetify` is explicitly versioned0.0.2. Its pending patch changeset is consumed and its original summary retained in the package changelog, avoiding an unintended second patch bump. Historical prerelease initial-version records remain historical records, not consumer dependencies.

No Core layout or dynamic plugin trait changes are introduced, so ABI0.5.4 remains. This does not authorize reuse of old binaries: the new cohort needs fresh Core/plugin builds, matching package-consumer tests and the affected Docs, Commerce, SSR and HMR checks. Source dependency exclusion is not yet a claim that every final tarball is qualified. D001 is considered avoided only after the final redistributed closure excludes Tardar; no original upstream license gap is represented as repaired.
