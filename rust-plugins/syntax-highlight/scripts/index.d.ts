export interface HighlightOptions {
  light?: 'inspired-github' | 'base16-ocean-dark';
  dark?: 'inspired-github' | 'base16-ocean-dark';
}
export default function syntaxHighlight(options?: HighlightOptions): [string, HighlightOptions];
