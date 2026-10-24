// cspell:words noembed
// Preserve generated code metadata at the SSR serialization boundary.
// This is deliberately not a general HTML sanitizer or parser.
const MARKER_ATTRIBUTE = 'data-farm-docs-code-block';
export const CODE_BLOCK_METADATA_MARKER = 'raw-cr-v1';
interface Attribute {
  name: string;
  start: number | null;
  end: number | null;
  quote: string | null;
}
interface Tag {
  name: string;
  closing: boolean;
  attributes: Attribute[];
  end: number;
  selfClosing: boolean;
}
interface RewriteResult {
  html: string;
  status: 'malformed' | 'encoded' | 'unchanged';
  reason?: string;
  attributes: number;
  carriageReturns: number;
}
const space = (character: string | undefined) =>
  character !== undefined && /[\t\n\f\r ]/.test(character);
const nameCharacter = (character: string | undefined) =>
  character !== undefined && /[A-Za-z0-9:_-]/.test(character);
const rawNames = new Set([
  'script',
  'style',
  'textarea',
  'title',
  'xmp',
  'iframe',
  'noembed',
  'noframes',
  'noscript'
]);

function tagAt(html: string, start: number): Tag | null {
  let index = start + 1;
  const closing = html[index] === '/';
  if (closing) index++;
  const beginning = index;
  if (!/[A-Za-z]/.test(html[index] ?? '')) return null;
  while (nameCharacter(html[index])) index++;
  const name = html.slice(beginning, index).toLowerCase();
  if (!space(html[index]) && !['/', '>'].includes(html[index]))
    throw new Error('unsupported tag name boundary');
  const attributes: Attribute[] = [];
  while (index < html.length) {
    while (space(html[index])) index++;
    if (html[index] === '>')
      return { name, closing, attributes, end: index + 1, selfClosing: false };
    if (html[index] === '/' && html[index + 1] === '>')
      return { name, closing, attributes, end: index + 2, selfClosing: true };
    if (closing) throw new Error('attributes on closing tag');
    const attributeStart = index;
    while (
      index < html.length &&
      !space(html[index]) &&
      !['=', '/', '>'].includes(html[index])
    ) {
      if (['\0', '"', "'", '<', '`'].includes(html[index]))
        throw new Error('invalid attribute name');
      index++;
    }
    if (index === attributeStart) throw new Error('invalid tag boundary');
    const attribute: Attribute = {
      name: html.slice(attributeStart, index).toLowerCase(),
      start: null,
      end: null,
      quote: null
    };
    while (space(html[index])) index++;
    if (html[index] === '=') {
      index++;
      while (space(html[index])) index++;
      if (html[index] === '"' || html[index] === "'") {
        attribute.quote = html[index++];
        attribute.start = index;
        while (index < html.length && html[index] !== attribute.quote) index++;
        if (index === html.length)
          throw new Error('unterminated quoted attribute');
        attribute.end = index++;
        if (!space(html[index]) && !['/', '>'].includes(html[index]))
          throw new Error('missing attribute separator');
      } else {
        attribute.start = index;
        while (
          index < html.length &&
          !space(html[index]) &&
          html[index] !== '>'
        ) {
          if (['\0', '"', "'", '<', '=', '`'].includes(html[index]))
            throw new Error('invalid unquoted attribute');
          index++;
        }
        attribute.end = index;
        if (attribute.start === attribute.end)
          throw new Error('missing attribute value');
      }
    }
    attributes.push(attribute);
  }
  throw new Error('unterminated tag');
}

/**
 * Preserve literal CR only inside quoted data-code of an explicitly owned pre.
 * The caller must reserve/protect the marker namespace. Matching a marker is
 * not authentication: exact author forgery is indistinguishable in HTML alone.
 * Malformed/unsupported input returns the ORIGINAL bytes with no partial edits.
 */
function rewriteOwnedPreMetadata(
  html: string,
  expectedMarker: string
): RewriteResult {
  if (
    typeof html !== 'string' ||
    !/^[A-Za-z0-9_-]{8,80}$/.test(expectedMarker ?? '')
  )
    throw new TypeError('Require HTML and an explicit literal reserved marker');
  const patches: {
    start: number;
    end: number;
    value: string;
    count: number;
  }[] = [];
  let index = 0,
    foreignDepth = 0;
  try {
    while (index < html.length) {
      const start = html.indexOf('<', index);
      if (start < 0) break;
      if (html.startsWith('<!--', start)) {
        const end = html.indexOf('-->', start + 4);
        if (end < 0) throw new Error('unterminated comment');
        // Unusual HTML comment endings have different parser recovery.
        if (html.slice(start + 4, end).includes('--!>'))
          throw new Error('ambiguous comment');
        index = end + 3;
        continue;
      }
      if (/^<!doctype(?:\s|>)/i.test(html.slice(start, start + 11))) {
        let cursor = start + 2;
        let quote: string | null = null;
        for (; cursor < html.length; cursor++) {
          const character = html[cursor];
          if (quote) {
            if (character === quote) quote = null;
          } else if (character === '"' || character === "'") quote = character;
          else if (character === '[')
            throw new Error('unsupported declaration subset');
          else if (character === '>') break;
        }
        if (cursor === html.length) throw new Error('unterminated declaration');
        index = cursor + 1;
        continue;
      }
      if (html.startsWith('<!', start) || html.startsWith('<?', start))
        throw new Error('unsupported declaration');
      const tag = tagAt(html, start);
      if (!tag) {
        index = start + 1;
        continue;
      }
      index = tag.end;
      if (tag.name === 'svg' || tag.name === 'math') {
        foreignDepth += tag.closing ? -1 : tag.selfClosing ? 0 : 1;
        if (foreignDepth < 0) throw new Error('unbalanced foreign content');
      }
      if (!tag.closing && tag.name === 'plaintext') break;
      if (!tag.closing && rawNames.has(tag.name)) {
        const close = new RegExp(`</${tag.name}(?=[\\t\\n\\f\\r />])`, 'ig');
        close.lastIndex = index;
        const match = close.exec(html);
        if (!match) throw new Error('unterminated raw-text element');
        if (
          tag.name === 'script' &&
          /<!--|<script(?=[\t\n\f\r />])/i.test(html.slice(index, match.index))
        )
          throw new Error('unsupported escaped script-data state');
        const endTag = tagAt(html, match.index);
        if (!endTag?.closing || endTag.name !== tag.name)
          throw new Error('malformed raw-text closing tag');
        index = endTag.end;
        continue;
      }
      if (tag.closing || tag.name !== 'pre' || foreignDepth) continue;
      const markers = tag.attributes.filter(
        (attribute) => attribute.name === MARKER_ATTRIBUTE
      );
      const owned = markers.some(
        (attribute) =>
          attribute.start !== null &&
          attribute.end !== null &&
          html.slice(attribute.start, attribute.end) === expectedMarker
      );
      if (!owned) continue;
      if (markers.length !== 1 || !markers[0].quote || tag.selfClosing)
        throw new Error('ambiguous owned pre marker');
      const values = tag.attributes.filter(
        (attribute) => attribute.name === 'data-code'
      );
      if (values.length !== 1 || !values[0].quote)
        throw new Error('owned pre requires one quoted data-code');
      const value = values[0];
      if (value.start === null || value.end === null)
        throw new Error('missing owned attribute value');
      const original = html.slice(value.start, value.end);
      if (original.includes('\0'))
        throw new Error('NUL is outside the serializer contract');
      if (original.includes('\r'))
        patches.push({
          start: value.start,
          end: value.end,
          value: original.replaceAll('\r', '&#13;'),
          count: original.split('\r').length - 1
        });
    }
    if (foreignDepth) throw new Error('unclosed foreign content');
  } catch (error) {
    return {
      html,
      status: 'malformed',
      reason: error instanceof Error ? error.message : String(error),
      attributes: 0,
      carriageReturns: 0
    };
  }
  let encoded = '',
    previous = 0;
  for (const patch of patches) {
    encoded += html.slice(previous, patch.start) + patch.value;
    previous = patch.end;
  }
  encoded += html.slice(previous);
  return {
    html: encoded,
    status: patches.length ? 'encoded' : 'unchanged',
    attributes: patches.length,
    carriageReturns: patches.reduce((total, patch) => total + patch.count, 0)
  };
}

/** Keep data-code equal to the original code before and after hydration.
 * No CR means no lexical scan. Marker ownership is a reserved naming contract,
 * not authentication; this transform preserves CR without decoding user markup.
 */
export function serializeCodeBlockMetadata(html: string): string {
  if (!html.includes('\r') || !html.includes(CODE_BLOCK_METADATA_MARKER))
    return html;
  const result = rewriteOwnedPreMetadata(html, CODE_BLOCK_METADATA_MARKER);
  if (result.status === 'malformed') {
    throw new Error(
      `[DOCS_SSR_CODE_METADATA] Cannot preserve code metadata: ${result.reason}`
    );
  }
  return result.html;
}
