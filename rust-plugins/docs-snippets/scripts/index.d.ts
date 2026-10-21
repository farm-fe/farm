export interface DocsSnippetsOptions {
  /** Presentation-only module exporting CodeBlock and CommandTabs. */
  rendererModule?: string;
  /** Rust highlighter themes. No runtime highlighting or JavaScript callbacks. */
  highlight?: {
    light?: 'inspired-github' | 'base16-ocean-dark';
    dark?: 'inspired-github' | 'base16-ocean-dark';
  };
}

export default function snippets(options?: DocsSnippetsOptions): [string, DocsSnippetsOptions];
