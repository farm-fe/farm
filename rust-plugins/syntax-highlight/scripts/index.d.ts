export interface HighlightOptions {
  light?: 'inspired-github' | 'base16-ocean-dark' | 'vscode-light' | 'vscode-dark';
  dark?: 'inspired-github' | 'base16-ocean-dark' | 'vscode-light' | 'vscode-dark';
}
export default function syntaxHighlight(options?: HighlightOptions): [string, HighlightOptions];
