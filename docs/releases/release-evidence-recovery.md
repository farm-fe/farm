# Release recovery boundary

Main release no longer waits for CI or Unit test evidence. The pending-evidence
controller and its workflow completion listener have been removed. No automatic
rerun or dispatch is performed. PR tests remain the merge quality gate.

Recovery stays in the original release transaction. Preserve the original
source, manifest, signed archive IDs and exact bytes after any partial upload.
A later attempt must verify that the previous owner ended, recheck the publication
lock and registry state, and resume only missing identical packages. A version
with different bytes is a conflict, never permission to rebuild or overwrite it.
An uncertain upload response requires a registry read before retrying.

The plan upload attempt and frozen candidate attempt remain independent. Fixed
outputs select their original artifacts; the current run number is not used to
guess an older upload. Missing or expired frozen proof fails closed.

A maintainer may use the existing original-run recovery procedure after inspecting
current state. GitHub's run-ID rerun endpoint has no attempt compare-and-swap;
there is no controller POST or recommendation that authorizes a future attempt.
Registry byte/tag verification, final Release confirmation and baseline advancement
remain required. These are publication integrity checks, not consumer tests.

Local simulations exercise production algorithms with simulated transports. They
do not prove hosted signing, scheduling, registry writes or successful publication.
