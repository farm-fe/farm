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
    return parsed;
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
  return str.replace(/\x1b\[[0-9;]*m/g, '');
}

export function cleanStack(stack: string) {
  return stack
    .split(/\n/g)
    .filter((l) => /^\s*at/.test(l))
    .join('\n');
}

export function splitErrorMessage(errorMsg: any) {
  // @ts-ignore
  const potentialCauses = errorMsg.cause;
  const stripErrorFrame = stripAnsi(errorMsg.errorFrame ?? '');
  const { codeBlocks, idCodeLines } = extractSwcCodeBlocks(stripErrorFrame);
  return {
    errorInfo: stripAnsi(errorMsg.message),
    codeBlocks,
    potentialCauses,
    idCodeLines,
    frame: stripErrorFrame,
    errorBrowser: `${stripAnsi(errorMsg.message)}\n\n${potentialCauses}\n\n`
  };
}

export function extractSwcCodeBlocks(errorMsg: string) {
  const lines = errorMsg.split('\n');
  let codeBlocks = [];
  let idCodeLines = [];
  let currentBlock: any = [];
  let inCodeBlock = false;
  let errorLine = '';
  let currentIdCodeLine = '';

  for (const line of lines) {
    if (line.trim().startsWith('×')) {
      if (inCodeBlock) {
        codeBlocks.push(currentBlock.join('\n'));
        idCodeLines.push(currentIdCodeLine);
        currentBlock = [];
        currentIdCodeLine = '';
      }
      errorLine = line;
      inCodeBlock = false;
    } else if (line.includes('╭─[')) {
      inCodeBlock = true;
      currentBlock = errorLine ? [errorLine, line] : [line];
      errorLine = '';
      const match = line.match(/\s*╭─(.*?)$$/);

      if (match) {
        currentIdCodeLine = match[1];
      }
    } else if (inCodeBlock) {
      currentBlock.push(line);
      if (line.includes('╰────')) {
        codeBlocks.push(currentBlock.join('\n'));
        idCodeLines.push(currentIdCodeLine);
        currentBlock = [];
        currentIdCodeLine = '';
        inCodeBlock = false;
      }
    }
  }

  if (currentBlock.length > 0) {
    codeBlocks.push(currentBlock.join('\n'));
    idCodeLines.push(currentIdCodeLine);
  }

  return { codeBlocks, idCodeLines };
}

export function extractErrorMessage(errorString: string) {
  const regex = /^([\s\S]*?)(?=\s+at constructor)/;

  const match = errorString.match(regex);

  return match ? match[1].trim() : errorString;
}
