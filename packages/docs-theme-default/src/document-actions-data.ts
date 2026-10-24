/** Accept only site-relative public paths; never fetch a remote origin. */
export function isDocumentActionUrl(value: unknown): value is string {
  if (
    typeof value !== 'string' ||
    !value.startsWith('/') ||
    value.startsWith('//') ||
    /[\\?#\x00-\x20\x7f]/u.test(value)
  )
    return false;
  return value
    .slice(1)
    .split('/')
    .every((part, index, parts) => {
      let decoded: string;
      try {
        decoded = decodeURIComponent(part);
      } catch {
        return false;
      }
      return (
        (Boolean(part) || index === parts.length - 1) &&
        !['.', '..'].includes(decoded) &&
        !/[\\/\s?#:\x00-\x1f\x7f]/u.test(decoded)
      );
    });
}

export async function readVerifiedMarkdown(
  url: string,
  expectedSha256: string,
  dependencies: { fetch: typeof fetch; digest: SubtleCrypto['digest'] } = {
    fetch: (...args) => fetch(...args),
    digest: (...args) => crypto.subtle.digest(...args)
  }
): Promise<string> {
  if (!isDocumentActionUrl(url) || !/^[a-f0-9]{64}$/.test(expectedSha256)) {
    throw new Error('[DOCS_AI_COPY] invalid Markdown identity');
  }
  const response = await dependencies.fetch(url, {
    headers: { Accept: 'text/markdown' },
    credentials: 'same-origin',
    redirect: 'error'
  });
  if (
    !response.ok ||
    !/^text\/(?:markdown|plain)(?:;|$)/i.test(
      response.headers.get('Content-Type') ?? ''
    )
  ) {
    throw new Error('[DOCS_AI_COPY] expected a Markdown response');
  }
  const bytes = await response.arrayBuffer();
  const digest = await dependencies.digest('SHA-256', bytes);
  const actual = Array.from(new Uint8Array(digest), (byte) =>
    byte.toString(16).padStart(2, '0')
  ).join('');
  if (actual !== expectedSha256)
    throw new Error('[DOCS_AI_COPY] content changed; reload the document');
  return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
}
