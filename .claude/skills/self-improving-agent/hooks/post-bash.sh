#!/usr/bin/env bash
set -euo pipefail

tool_output="${1:-}"
exit_code="${2:-0}"

echo "[self-improving-agent] PostToolUse: exit=${exit_code}" >&2
if [[ -n "${tool_output}" ]]; then
  echo "[self-improving-agent] Output: ${tool_output}" >&2
fi
