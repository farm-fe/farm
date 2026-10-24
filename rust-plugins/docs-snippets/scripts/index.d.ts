export interface DocsSnippetsOptions {
  /** Presentation-only module exporting CodeBlock and CommandTabs. */
  rendererModule?: string;
  /** Rust highlighter themes. No runtime highlighting or JavaScript callbacks. */
  highlight?: {
    light?: 'inspired-github' | 'base16-ocean-dark' | 'vscode-light' | 'vscode-dark';
    dark?: 'inspired-github' | 'base16-ocean-dark' | 'vscode-light' | 'vscode-dark';
    /** Default jsx. Static HTML requires custom code components to forward props. */
    representation?: 'jsx' | 'static-html';
  };
}

export default function snippets(options?: DocsSnippetsOptions): [string, DocsSnippetsOptions];
