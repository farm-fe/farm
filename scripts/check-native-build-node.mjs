// Adapted from bc577f8d: fail before native work when required tool engines are incompatible.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { collectInstalledDependencies, evaluateNodeEngines } from './v2-node-engines.mjs';

export function auditNativeBuildNode(node, graph) {
  const result = evaluateNodeEngines(graph.packages, [node])[0];
  const missingRequired = graph.missing.filter(item => !item.optional);
  return { ...result, packageCount: graph.packages.length, success: graph.packages.length > 0 && result.satisfiesInstalledRanges && missingRequired.length === 0, missingRequired };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const directory = fileURLToPath(new URL('../packages/plugin-tools/', import.meta.url));
    const report = auditNativeBuildNode(process.version, collectInstalledDependencies(directory));
    console.log(JSON.stringify(report));
    if (!report.success) {
      console.error('Native build requires a Node runtime compatible with the installed required plugin-tools dependencies. Review engines and missing dependencies before building.');
      process.exitCode = 1;
    }
  } catch (error) {
    console.error(`Could not inspect the native build toolchain: ${error.message}`);
    process.exitCode = 1;
  }
}
