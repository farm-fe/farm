import React from 'react';
import { renderToStaticMarkup, renderToString } from 'react-dom/server';
import Content, { frontmatter, toc, mappedFailure } from './sample.mdx';
import CommentContent, { toc as commentToc } from './comments.md';
import Boundaries from './boundaries.mdx';
import HtmlContent from './html.md';
export { frontmatter, toc };
export { mappedFailure };
export { commentToc };
export function render() { return renderToString(<Content />); }
export function renderComments() { return renderToStaticMarkup(<CommentContent />); }
export function renderBoundaries() { return renderToStaticMarkup(<Boundaries />); }
export function renderHtml() { return renderToStaticMarkup(<HtmlContent />); }
