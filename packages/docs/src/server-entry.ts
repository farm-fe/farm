import {
  renderNotFound as renderNotFoundElement,
  renderPage,
  snapshotId
} from 'docs:application';
import { createElement } from 'react';
import { renderToString } from 'react-dom/server';
import { serializeCodeBlockMetadata } from './code-block-metadata.js';
import { RuntimeFrame } from './runtime-frame.js';
export { snapshotId };
export function render(key: string): string {
  return serializeCodeBlockMetadata(
    renderToString(createElement(RuntimeFrame, { content: renderPage(key) }))
  );
}
export function renderNotFound(locale: string): string {
  return serializeCodeBlockMetadata(
    renderToString(
      createElement(RuntimeFrame, { content: renderNotFoundElement(locale) })
    )
  );
}
