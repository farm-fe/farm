import { createHash } from 'node:crypto';

export interface MachineReadablePage {
  key: string;
  markdownPath: string;
  markdownUrl: string;
  sha256: string;
}

export interface MachineReadableManifest {
  schemaVersion: 1;
  snapshotId: string;
  manifestPath: 'docs-ai-manifest.json';
  indexPath: 'llms.txt';
  pages: MachineReadablePage[];
}

interface PublicPageIdentity {
  key: string;
  title: string;
  locale: string;
  version?: string | null;
  markdownUrl?: string | null;
  markdownSha256?: string | null;
  canonicalUrl: string;
  headings: { id: string; title: string; depth: number }[];
}

export interface MachineReadableContext {
  snapshotId: string;
  basePath: string;
  enabled: boolean;
  publicationStatus: 'prerelease' | 'stable';
  pages: PublicPageIdentity[];
}

const invalid = (detail: string) => new Error(`[DOCS_AI_RESOURCE] ${detail}`);
const hash = (bytes: Buffer) =>
  createHash('sha256').update(bytes).digest('hex');
const sha256 = (value: unknown): value is string =>
  typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);

function object(
  value: unknown,
  fields: string[],
  name: string
): Record<string, unknown> {
  if (
    !value ||
    typeof value !== 'object' ||
    Array.isArray(value) ||
    Object.keys(value).sort().join(',') !== fields.sort().join(',')
  ) {
    throw invalid(`invalid ${name} fields`);
  }
  return value as Record<string, unknown>;
}

/** Validate names before URL normalization can erase traversal segments. */
function resourcePath(value: unknown): value is string {
  if (
    typeof value !== 'string' ||
    !value ||
    value.startsWith('/') ||
    /[\\\s?#:\x00-\x1f\x7f]/u.test(value)
  )
    return false;
  return value.split('/').every((part) => {
    let decoded: string;
    try {
      decoded = decodeURIComponent(part);
    } catch {
      return false;
    }
    return (
      Boolean(part) &&
      !['.', '..'].includes(decoded) &&
      !/[\\/\s?#:\x00-\x1f\x7f]/u.test(decoded)
    );
  });
}

function publicUrl(
  value: unknown,
  basePath: string,
  resource: string
): value is string {
  if (
    typeof value !== 'string' ||
    !value.startsWith('/') ||
    value.startsWith('//') ||
    /[\\?#\x00-\x20\x7f]/u.test(value)
  )
    return false;
  const expected = new URL(basePath + resource, 'https://farm-docs.invalid');
  const actual = new URL(value, 'https://farm-docs.invalid');
  return (
    actual.origin === expected.origin && actual.pathname === expected.pathname
  );
}

function json(bytes: Buffer, name: string): unknown {
  try {
    return JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch {
    throw invalid(`${name} must be UTF-8 JSON`);
  }
}

/** Validate native output only. Content generation and final sealing remain native. */
export function validateMachineReadableResources(
  value: unknown,
  context: MachineReadableContext,
  resources: Record<string, Buffer>,
  declaredNames: readonly string[]
): MachineReadableManifest | undefined {
  const expected = new Map(
    context.pages
      .filter((page) => page.markdownUrl)
      .map((page) => [page.key, page])
  );
  if (value === undefined || value === null) {
    if (context.enabled || expected.size)
      throw invalid('enabled output is missing');
    return;
  }
  if (!context.enabled)
    throw invalid('output requires explicit AI configuration');
  const summary = object(
    value,
    ['schemaVersion', 'snapshotId', 'manifestPath', 'indexPath', 'pages'],
    'summary'
  );
  if (
    summary.schemaVersion !== 1 ||
    !sha256(summary.snapshotId) ||
    summary.snapshotId !== context.snapshotId ||
    summary.manifestPath !== 'docs-ai-manifest.json' ||
    summary.indexPath !== 'llms.txt' ||
    !Array.isArray(summary.pages)
  ) {
    throw invalid('schema, snapshot or index identity differs');
  }
  const declared = new Set(declaredNames);
  const names = new Set<string>();
  const member = (name: unknown): Buffer => {
    if (
      !resourcePath(name) ||
      !declared.has(name) ||
      !Object.hasOwn(resources, name) ||
      names.has(name.toLowerCase())
    ) {
      throw invalid('unsafe, missing or duplicate output member');
    }
    names.add(name.toLowerCase());
    return resources[name];
  };
  const manifestBytes = member(summary.manifestPath);
  const index = member(summary.indexPath);
  try {
    new TextDecoder('utf-8', { fatal: true }).decode(index);
  } catch {
    throw invalid('index must be UTF-8');
  }
  const keys = new Set<string>();
  const pages = summary.pages.map((entry) => {
    const page = object(
      entry,
      ['key', 'markdownPath', 'markdownUrl', 'sha256'],
      'page summary'
    );
    if (
      typeof page.key !== 'string' ||
      keys.has(page.key) ||
      !resourcePath(page.markdownPath) ||
      !page.markdownPath.endsWith('.md') ||
      !publicUrl(page.markdownUrl, context.basePath, page.markdownPath) ||
      !sha256(page.sha256)
    ) {
      throw invalid('invalid page identity');
    }
    const target = expected.get(page.key);
    if (
      !target ||
      target.markdownUrl !== page.markdownUrl ||
      target.markdownSha256 !== page.sha256
    ) {
      throw invalid('page and Markdown identities differ');
    }
    const bytes = member(page.markdownPath);
    if (hash(bytes) !== page.sha256)
      throw invalid('Markdown bytes differ from declared hash');
    try {
      new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    } catch {
      throw invalid('Markdown must be UTF-8');
    }
    keys.add(page.key);
    return page as unknown as MachineReadablePage;
  });
  if (keys.size !== expected.size)
    throw invalid('public Markdown page is missing from output');
  const manifest = object(
    json(manifestBytes, 'manifest'),
    ['schemaVersion', 'snapshotId', 'publicationStatus', 'pages'],
    'public manifest'
  );
  if (
    manifest.schemaVersion !== 1 ||
    manifest.snapshotId !== context.snapshotId ||
    manifest.publicationStatus !== context.publicationStatus ||
    !Array.isArray(manifest.pages) ||
    manifest.pages.length !== pages.length
  ) {
    throw invalid('public manifest and build snapshot differ');
  }
  const publicKeys = new Set<string>();
  for (const value of manifest.pages) {
    const page = object(
      value,
      [
        'key',
        'title',
        'locale',
        'version',
        'canonicalUrl',
        'markdownPath',
        'markdownUrl',
        'sha256',
        'headings'
      ],
      'public page'
    );
    const target = context.pages.find(
      (candidate) => candidate.key === page.key
    );
    const output = pages.find((candidate) => candidate.key === page.key);
    if (
      !target ||
      !output ||
      publicKeys.has(output.key) ||
      page.title !== target.title ||
      page.locale !== target.locale ||
      page.version !== (target.version ?? null) ||
      page.markdownPath !== output.markdownPath ||
      page.markdownUrl !== output.markdownUrl ||
      page.sha256 !== output.sha256 ||
      page.canonicalUrl !== target.canonicalUrl ||
      !Array.isArray(page.headings) ||
      page.headings.length !== target.headings.length
    )
      throw invalid('public page metadata differs');
    for (const [index, heading] of page.headings.entries()) {
      const item = object(heading, ['id', 'title', 'depth'], 'public heading');
      if (
        typeof item.id !== 'string' ||
        typeof item.title !== 'string' ||
        !Number.isInteger(item.depth) ||
        (item.depth as number) < 1 ||
        (item.depth as number) > 6
      )
        throw invalid('invalid public heading');
      const expected = target.headings[index];
      if (
        item.id !== expected.id ||
        item.title !== expected.title ||
        item.depth !== expected.depth
      ) {
        throw invalid('public page metadata differs');
      }
    }
    publicKeys.add(output.key);
  }
  return summary as unknown as MachineReadableManifest;
}
