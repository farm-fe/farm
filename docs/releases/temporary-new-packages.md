# Temporary publication of new npm package names

This one-time manual workflow bootstraps the 48 new `0.0.1` package names in
PR #2351's v2 release. Remove it after the bootstrap. It does not replace the
normal trusted-publishing release, create a GitHub Release, or promote tags on
existing packages.

The workflow is dispatched from `main` with the confirmation input selected.
It uses the repository secret `NPM_BYPASS_2FA_TOKEN` only for the npm publish
subprocess. The token must already have the appropriate npm package-creation and
publish permissions; never put it in workflow inputs, logs, or repository files.
The workflow has no OIDC permission and does not change npm account security.

It consumes only the original signed candidates from run `42964546726`, attempt
`1`, source `74a09f11e189ac50bfe193885a910cbf264eefd8`. The original run must
finish first. The full 297-package manifest, source plan, seal, original
candidate artifact identity and archive hashes are checked before the 48-name
allowlist is selected. A missing, expired, unsigned or changed artifact stops
publication; do not rebuild replacement archives under this procedure.

Each allowed name is checked at the public registry immediately before upload.
A package name that already exists is never uploaded or tagged again. A matching
`0.0.1` archive is reported as already published only after byte verification;
other existing names and registry failures are reported as failures. npm's
immutable version protection prevents overwriting an existing version. npm has
no atomic create-if-name-absent operation, so another publisher could create a
name between the final check and upload; coordinate external publishers.

New archives are published unchanged with scripts disabled and tag
`farm-release-0.0.1`. The flow does not certify dependency availability or set
`latest` on existing packages. Review the per-package results artifact, including
uncertain upload outcomes, before retrying. A failed run does not imply that no
package was published. Never retry an uncertain upload without reading back the
registry. A successful report is not a website build or deployment; validate the
complete published consumer separately.
