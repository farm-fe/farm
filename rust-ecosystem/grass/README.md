# Farm Grass backend

Derived from `grass_compiler` 0.13.4, upstream commit
`e0bb9e2eabfc3a58e42b03089cd7b22c68d09d0b` (MIT).
`UPSTREAM.json` records the original source hashes before Farm modifications.

This backend exists to retain Sass source positions through serialization and
resolve dotted, extensionless Sass module names correctly. It does not execute
JavaScript or invoke another compiler. The official Rust Sass plugin consumes
this crate directly; it is not a separate user-facing Sass implementation.

Keep changes focused on source maps and import resolution. Upstream source uses
edition 2021. Preserve the original license in source and binary distributions.
