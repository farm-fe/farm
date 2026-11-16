// cspell:words noembed
import assert from 'node:assert/strict';

// Public ownership proof, not an Algolia API key. Keep it out of package templates:
// only this website's published-package deployment homepage needs the marker.
const name = 'algolia-site-verification';
const value = 'C8B1581AB301BA8E';
const marker = `<meta name="${name}" content="${value}" />`;
const check = (condition, message) => assert.ok(condition, `[DOCS_SITE_VERIFICATION] ${message}`);

function homepageTags(html) {
  const tags = [];
  // Preserve offsets, ignore comments and raw text, and allow '>' in attributes.
  const token = /<!--[^]*?-->|<![^>]*>|<\/?([a-z][\w:-]*)\b(?:[^<>"']|"[^"]*"|'[^']*')*>/gi;
  let match;
  while ((match = token.exec(html))) {
    if (!match[1]) continue;
    const tag = { name: match[1].toLowerCase(), closing: match[0][1] === '/', start: match.index, end: token.lastIndex, text: match[0] };
    tags.push(tag);
    if (!tag.closing && /^(script|style|title|textarea|xmp|iframe|noembed|noframes)$/.test(tag.name)) {
      const close = new RegExp(`</${tag.name}\\s*>`, 'gi');
      close.lastIndex = token.lastIndex;
      const end = close.exec(html);
      check(end, `Unclosed ${tag.name} element`);
      token.lastIndex = close.lastIndex;
      tag.end = close.lastIndex;
    }
  }
  return tags;
}

function homepageHead(html, tags) {
  const roots = tags.filter(tag => tag.name === 'html');
  check(roots.length >= 1 && roots.length <= 2 && !roots[0].closing && (roots.length === 1 || roots[1].closing), 'Expected one html root');
  check(!html.slice(0, roots[0].start).replace(/<!--[^]*?-->|<!doctype html\s*>/gi, '').trim(), 'Unexpected content before html');
  const heads = tags.filter(tag => tag.name === 'head');
  const bodies = tags.filter(tag => tag.name === 'body');
  if (heads.length || bodies.length) {
    check(heads.length === 2 && !heads[0].closing && heads[1].closing && heads[0].start >= roots[0].end, 'Expected exactly one complete head');
    check(bodies.length === 2 && !bodies[0].closing && bodies[1].closing && bodies[0].start >= heads[1].end, 'Expected body after head');
    check(!html.slice(roots[0].end, heads[0].start).replace(/<!--[^]*?-->/g, '').trim(), 'Unexpected content before head');
    check(!html.slice(heads[1].end, bodies[0].start).replace(/<!--[^]*?-->/g, '').trim(), 'Unexpected content before body');
    check(!roots[1] || roots[1].start >= bodies[1].end, 'Invalid html end boundary');
    validateHeadPrefix(html, tags, heads[0].end, heads[1].start);
    return { start: heads[0].end, end: heads[1].start };
  }
  // Farm's minifier omits optional head/body tags. Accept only the published
  // template's known head-only prefix ending at its unique content root. A meta
  // immediately after <html> belongs to the implicit head, without rewriting HTML.
  const content = tags.filter(tag => tag.name === 'div' && !tag.closing && attributes(tag).some(([key, val]) => key === 'id' && val === '__FARM_DOCS_ROOT__'));
  check(content.length === 1 && content[0].start >= roots[0].end, 'Missing or duplicate Farm homepage root');
  validateHeadPrefix(html, tags, roots[0].end, content[0].start);
  check(!roots[1] || (roots[1].start > content[0].end && !html.slice(roots[1].end).trim()), 'Invalid html end boundary');
  return { start: roots[0].end, end: content[0].start };
}

function validateHeadPrefix(html, tags, start, end) {
  let offset = start;
  for (const tag of tags.filter(tag => tag.start >= offset && tag.start < end)) {
    check(!tag.closing && /^(base|link|meta|title|style|script)$/.test(tag.name), 'Unexpected element before homepage body');
    check(!html.slice(offset, tag.start).replace(/<!--[^]*?-->/g, '').trim(), 'Unexpected text before homepage body');
    offset = tag.end;
  }
  check(!html.slice(offset, end).replace(/<!--[^]*?-->/g, '').trim(), 'Unexpected text before homepage body');
}

function attributes(tag) {
  return [...tag.text.replace(/^<[^\s/>]+/, '').slice(0, -1).matchAll(/([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g)].map(match => [match[1].toLowerCase(), match[2] ?? match[3] ?? match[4] ?? '']);
}

function inspectHomepage(html) {
  check(typeof html === 'string', 'Homepage must be HTML text');
  const tags = homepageTags(html);
  const head = homepageHead(html, tags);
  const markers = [];
  for (const tag of tags.filter(tag => tag.name === 'meta' && !tag.closing)) {
    const entries = attributes(tag);
    check(!entries.some(([key, val]) => key === 'name' && val.includes('&')), 'Unsupported encoded meta name');
    if (!entries.some(([key, val]) => key === 'name' && val.toLowerCase() === name)) continue;
    check(new Set(entries.map(([key]) => key)).size === entries.length, 'Duplicate verification attributes');
    const attrs = Object.fromEntries(entries);
    check(attrs.name === name && attrs.content === value && entries.length === 2, 'Conflicting verification marker');
    check(tag.start >= head.start && tag.end <= head.end, 'Verification marker must be in head');
    markers.push(tag);
  }
  check(markers.length <= 1, 'Duplicate verification markers');
  return { insertAt: head.start, markers };
}

export function assertHomepageVerification(html) {
  check(inspectHomepage(html).markers.length === 1, 'Missing verification marker');
}

export function addHomepageVerification(html) {
  check(typeof html === 'string', 'Homepage must be HTML text');
  const { insertAt, markers } = inspectHomepage(html);
  const result = markers.length ? html : html.slice(0, insertAt) + marker + html.slice(insertAt);
  assertHomepageVerification(result);
  return result;
}
