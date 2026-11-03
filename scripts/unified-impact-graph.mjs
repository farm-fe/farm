import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// There is one dependency parser. JS consumers project its typed edges rather
// than separately interpreting workspace aliases or Cargo dependency kinds.
export function unifiedGraph(snapshots) {
  return JSON.parse(execFileSync(process.env.FARM_GRAPH_PYTHON ?? 'python3',
    ['-B', fileURLToPath(new URL('./unified-impact-graph.py', import.meta.url))], {
      input: JSON.stringify({ snapshots }), encoding: 'utf8', timeout: 30000,
      maxBuffer: 32 * 1024 ** 2
    }));
}
