import { parse } from '@babel/parser';

type Statement = ReturnType<typeof parse>['program']['body'][number];
type VariableDeclaration = Extract<Statement, { type: 'VariableDeclaration' }>;

// loader-shared expects one resolver per declaration. Split only statements,
// keeping loop headers and unrelated declarations intact.
export function normalizeResolverDeclarations(content: string): string {
  const ast = parse(content, {
    sourceType: 'module',
    plugins: ['typescript', 'jsx'],
    tokens: true
  });
  const edits: { start: number; end: number; replacement: string }[] = [];

  function visit(value: unknown, parentType?: string) {
    if (!value || typeof value !== 'object') return;
    if (Array.isArray(value)) {
      for (const child of value) visit(child, parentType);
      return;
    }
    const node = value as Record<string, unknown>;
    if (
      node.type === 'VariableDeclaration' &&
      ['Program', 'BlockStatement', 'SwitchCase'].includes(parentType ?? '')
    ) {
      const declaration = value as VariableDeclaration;
      if (
        declaration.declarations.length > 1 &&
        declaration.declarations.some(
          ({ init }) =>
            init?.type === 'CallExpression' &&
            init.callee.type === 'Identifier' &&
            ['_resolveComponent', '_resolveDirective'].includes(
              init.callee.name
            )
        )
      ) {
        for (let index = 1; index < declaration.declarations.length; index++) {
          const before = declaration.declarations[index - 1];
          const after = declaration.declarations[index];
          const beforeEnd = before.end;
          const afterStart = after.start;
          if (beforeEnd == null || afterStart == null) {
            throw new Error('Vue resolver declaration has no source range');
          }
          const separator = ast.tokens?.find(
            (token) =>
              token.type.label === ',' &&
              token.start >= beforeEnd &&
              token.end <= afterStart
          );
          if (!separator)
            throw new Error('Vue resolver declaration has no separator');
          edits.push({
            start: separator.start,
            end: separator.end,
            replacement: ';\n'
          });
          edits.push({
            start: afterStart,
            end: afterStart,
            replacement: `${declaration.kind} `
          });
        }
      }
    }
    for (const [key, child] of Object.entries(node)) {
      if (key !== 'loc' && key !== 'comments' && key !== 'tokens') {
        visit(child, typeof node.type === 'string' ? node.type : parentType);
      }
    }
  }
  visit(ast);
  for (const edit of edits.sort((a, b) => b.start - a.start)) {
    content =
      content.slice(0, edit.start) + edit.replacement + content.slice(edit.end);
  }
  return content;
}
