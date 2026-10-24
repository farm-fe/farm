export type CommandTool = 'npm' | 'pnpm' | 'yarn' | 'bun';
export type CodeBlockProps = {
  language?: string;
  title?: string;
  showLineNumbers?: boolean;
  highlightLines?: string;
} & ({ code: string; children?: never } | { code?: never; children?: string });
export interface CommandTabsProps {
  commands: Partial<Record<CommandTool, string>>;
}

/** Compile-time marker. Rust replaces this import and compiles static props. */
export function CodeBlock(_props: CodeBlockProps): never {
  throw new Error(
    '[DOCS_SNIPPET_UNCOMPILED] CodeBlock requires @farmfe/plugin-docs-snippets'
  );
}

/** Commands are explicit: no runtime shell rewriting or highlighting. */
export function CommandTabs(_props: CommandTabsProps): never {
  throw new Error(
    '[DOCS_SNIPPET_UNCOMPILED] CommandTabs requires @farmfe/plugin-docs-snippets'
  );
}
