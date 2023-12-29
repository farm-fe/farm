import type { RollupError } from 'rollup';
import { colors } from '../utils/color.js';
import { pad } from '../utils/share.js';
// import { DevServer } from './index.js';

export function prepareError(err: Error & { potentialSolution?: string }) {
  return {
    message: stripAnsi(err.message),
    stack: stripAnsi(cleanStack(err.stack || '')),
    id: (err as RollupError).id,
    frame: stripAnsi((err as RollupError).frame || ''),
    plugin: (err as RollupError).plugin,
    pluginCode: (err as RollupError).pluginCode?.toString(),
    loc: (err as RollupError).loc,
    potential: err.potentialSolution || ''
  };
}

export function stripAnsi(str: string) {
  // eslint-disable-next-line no-control-regex
  return str.replace(/\x1b\[[0-9;]*m/g, '');
}

export function cleanStack(stack: string) {
  return stack
    .split(/\n/g)
    .filter((l) => /^\s*at/.test(l))
    .join('\n');
}

export function buildErrorMessage(
  err: RollupError,
  args: string[] = [],
  includeStack = true
): string {
  if (err.plugin) args.push(`  Plugin: ${colors.magenta(err.plugin)}`);
  const loc = err.loc ? `:${err.loc.line}:${err.loc.column}` : '';
  if (err.id) args.push(`  File: ${colors.cyan(err.id)}${loc}`);
  if (err.frame) args.push(colors.yellow(pad(err.frame)));
  if (includeStack && err.stack) args.push(pad(cleanStack(err.stack)));
  return args.join('\n');
}

// export function logError(server: DevServer, err: RollupError): void {
//   const msg = buildErrorMessage(err, [
//     colors.red(`Internal server error: ${err.message}`)
//   ]);

//   server.config.logger.error(msg, {
//     clear: true,
//     timestamp: true,
//     error: err
//   });

//   server.ws.send({
//     type: 'error',
//     err: prepareError(err)
//   });
// }
