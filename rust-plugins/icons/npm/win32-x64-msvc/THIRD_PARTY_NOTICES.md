# Third-party notices

This native plugin links the following upstream implementations:

- Fervid Vue compiler crates: https://github.com/phoenix-ru/fervid at
  4f26ab48f81eb9688fb84e9fa14541373d5bca7d, Apache-2.0.
  Author named in the crate manifests: Marsel Shaikhin. Original license:
  licenses/fervid-Apache-2.0.txt. The pinned repository has no separate NOTICE.
  Farm uses its local fervid_parser patch for case-preserving Vue tags; source
  modifications are described in patches/fervid_parser/README.farm.md.
- The linked Farm svgr-rs implementation retains the MIT license and notices
  of Cong-Cong Pan and Farm Authors in licenses/svgr-rs-MIT.txt (canonical LF
  copy of rust-ecosystem/svgr-rs/LICENSE).

The package's LICENSE preserves the repository MIT terms for its wrapper.
These notices do not replace upstream terms or claim a complete transitive
compiler dependency audit. Icon collection data has its own licenses; this
package does not grant a license for arbitrary external collections.
