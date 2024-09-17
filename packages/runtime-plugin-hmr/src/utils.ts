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

export function splitErrorMessage(errorMsg: any) {
  // const potentialCausesRegex = /Potential Causes:[\s\S]*$/;

  // const potentialCausesMatch = errorMsg.match(potentialCausesRegex);
  // let potentialCauses = "";
  // if (potentialCausesMatch) {
  // const causes = potentialCausesMatch[0].split("\n\n")[0].trim();
  // potentialCauses = causes;
  // }

  // let errorInfo = errorMsg.replace(potentialCausesRegex, "").trim();
  // @ts-ignore
  const potentialCauses = errorMsg.cause;
  return {
    errorInfo: stripAnsi(errorMsg.message),
    codeBlocks: extractCodeBlocks(stripAnsi(errorMsg.frame)),
    potentialCauses,
    errorBrowser: `${stripAnsi(errorMsg.message)}\n\n${potentialCauses}\n\n`
  };
}

export function extractCodeBlocks(errorMsg: string) {
  const lines = errorMsg.split('\n');
  let codeBlocks = [];
  let currentBlock: any = [];
  let inCodeBlock = false;
  let errorLine = '';

  for (const line of lines) {
    if (line.trim().startsWith('×')) {
      // 如果当前有正在处理的代码块，先保存它
      if (inCodeBlock) {
        codeBlocks.push(currentBlock.join('\n'));
        currentBlock = [];
      }
      errorLine = line;
      inCodeBlock = false;
    } else if (line.includes('╭─[')) {
      inCodeBlock = true;
      currentBlock = errorLine ? [errorLine, line] : [line];
      errorLine = '';
    } else if (inCodeBlock) {
      currentBlock.push(line);
      if (line.includes('╰────')) {
        codeBlocks.push(currentBlock.join('\n'));
        currentBlock = [];
        inCodeBlock = false;
      }
    }
  }

  if (currentBlock.length > 0) {
    codeBlocks.push(currentBlock.join('\n'));
  }

  return codeBlocks;
}
