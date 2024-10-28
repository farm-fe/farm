// 定义基本的语法规则
const KEYWORDS =
  /\b(?:as|async|await|break|case|catch|class|const|continue|debugger|default|delete|do|else|enum|export|extends|finally|for|from|function|get|if|implements|import|in|instanceof|interface|let|new|null|of|package|private|protected|public|return|set|static|super|switch|this|throw|try|typeof|undefined|var|void|while|with|yield)\b/;
const FUNCTION = /\b[A-Za-z$_][0-9A-Za-z$_]*\b(?=\s*\()/;
const NUMBER = /\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b/i;
const STRING = /(["'`])(?:(?!\1)[^\\\n]|\\.)*\1/;
const COMMENT = /\/\/.*|\/\*[\s\S]*?\*\//;
const PUNCTUATION = /[{}[\];(),.:]/;

// 定义标记类型
type Token = {
  type: string;
  content: string;
};

export function highlight(code: string): string {
  const tokens: Token[] = [];
  let rest = code;

  while (rest) {
    // 空白字符
    const whiteSpace = rest.match(/^\s+/);
    if (whiteSpace) {
      tokens.push({ type: 'whitespace', content: whiteSpace[0] });
      rest = rest.slice(whiteSpace[0].length);
      continue;
    }

    // 注释
    const comment = rest.match(COMMENT);
    if (comment) {
      tokens.push({ type: 'comment', content: comment[0] });
      rest = rest.slice(comment[0].length);
      continue;
    }

    // 字符串
    const string = rest.match(STRING);
    if (string) {
      tokens.push({ type: 'string', content: string[0] });
      rest = rest.slice(string[0].length);
      continue;
    }

    // 关键字
    const keyword = rest.match(KEYWORDS);
    if (keyword) {
      tokens.push({ type: 'keyword', content: keyword[0] });
      rest = rest.slice(keyword[0].length);
      continue;
    }

    // 函数
    const func = rest.match(FUNCTION);
    if (func) {
      tokens.push({ type: 'function', content: func[0] });
      rest = rest.slice(func[0].length);
      continue;
    }

    // 数字
    const number = rest.match(NUMBER);
    if (number) {
      tokens.push({ type: 'number', content: number[0] });
      rest = rest.slice(number[0].length);
      continue;
    }

    // 标点符号
    const punctuation = rest.match(PUNCTUATION);
    if (punctuation) {
      tokens.push({ type: 'punctuation', content: punctuation[0] });
      rest = rest.slice(punctuation[0].length);
      continue;
    }

    // 其他字符
    tokens.push({ type: 'plain', content: rest[0] });
    rest = rest.slice(1);
  }

  // 将标记转换为HTML
  return tokens
    .map((token) => {
      if (token.type === 'whitespace') {
        return token.content;
      }
      return `<span class="token ${token.type}">${escapeHtml(token.content)}</span>`;
    })
    .join('');
}

// HTML转义
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// 导出一个类似 highlight.js 的接口
export const miniPrism = {
  highlight: (code: string) => highlight(code),
  highlightAuto: (code: string) => ({
    value: highlight(code)
  })
};
