export function handleErrorSync(
  fun: (...args: any[]) => void,
  args: any[],
  cb: (err: Error) => void = noop
) {
  try {
    fun(...args);
  } catch (err) {
    cb(err);
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-function
function noop() {}

export function parseIfJSON(str: string): any {
  if (typeof str !== 'string') {
    return str;
  }

  try {
    const parsed = JSON.parse(str);
    if (Object(parsed) !== parsed) {
      return str;
    }
    return prepareError(parsed);
  } catch (e) {
    return str;
  }
}

// remove rollup deps use any type
export function prepareError(err: Error & { potentialSolution?: string }) {
  return {
    message: stripAnsi(err.message),
    stack: stripAnsi(cleanStack(err.stack || '')),
    id: (err as any).id,
    frame: stripAnsi((err as any).frame || ''),
    plugin: (err as any).plugin,
    pluginCode: (err as any).pluginCode?.toString(),
    loc: (err as any).loc,
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

export function splitErrorMessage(errorMsg: string) {
  const potentialCausesRegex = /Potential Causes:[\s\S]*$/;

  const potentialCausesMatch = errorMsg.match(potentialCausesRegex);
  let potentialCauses = '';
  if (potentialCausesMatch) {
    const causes = potentialCausesMatch[0].split('\n\n')[0].trim();
    potentialCauses = causes;
  }

  let errorInfo = errorMsg.replace(potentialCausesRegex, '').trim();

  return {
    errorInfo: stripAnsi(errorInfo),
    codeBlocks: extractCodeBlocks(stripAnsi(errorInfo)),
    potentialCauses,
    errorBrowser: `${errorInfo}\n\n${potentialCauses}\n\n`
  };
}

export function extractCodeBlocks(errorMsg: any) {
  const lines = errorMsg.split('\n');
  let codeBlocks = [];
  let currentBlock = [];
  let inCodeBlock = false;

  for (const line of lines) {
    if (line.includes('╭─[')) {
      inCodeBlock = true;
      currentBlock = [line];
    } else if (inCodeBlock) {
      currentBlock.push(line);
      if (line.includes('╰────')) {
        codeBlocks.push(currentBlock.join('\n'));
        inCodeBlock = false;
      }
    }
  }

  return codeBlocks;
}
