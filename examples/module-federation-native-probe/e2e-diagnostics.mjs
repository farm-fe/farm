import assert from "node:assert/strict";

// A diagnostic printed before a hang or crash is not a completed rejection.
export function assertDiagnosticFailure(result, expected) {
  if (result.error) throw result.error;
  assert.equal(result.signal, null, "Compiler must exit normally, not by signal");
  assert.ok(Number.isInteger(result.status), "Compiler must have an exit code");
  assert.notEqual(result.status, 0, "Compiler must reject the invalid input");
  const diagnostic = result.stdout + result.stderr;
  assert.match(diagnostic, expected);
  return diagnostic;
}
