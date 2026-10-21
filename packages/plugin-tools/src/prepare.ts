import fs from 'node:fs/promises';
import path from 'node:path';

/** Prepare reviewable platform manifests without running publication commands. */
export async function preparePlatformPackages(
  root = process.cwd(),
  write = false
) {
  const parentPath = path.join(root, 'package.json');
  const parent = JSON.parse(await fs.readFile(parentPath, 'utf8'));
  let license: string | undefined;
  try {
    license = await fs.readFile(path.join(root, 'LICENSE'), 'utf8');
    if (!license.trim()) throw new Error('Package LICENSE must not be empty');
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
  }
  const notices: { relative: string; bytes: Buffer }[] = [];
  async function collectNotices(relative: string): Promise<void> {
    const file = path.join(root, relative);
    let stat;
    try {
      stat = await fs.lstat(file);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') return;
      throw error;
    }
    if (stat.isSymbolicLink())
      throw new Error('Notice symlinks are unsupported');
    if (stat.isDirectory()) {
      for (const entry of (await fs.readdir(file)).sort()) {
        await collectNotices(path.join(relative, entry));
      }
    } else if (stat.isFile()) {
      notices.push({ relative, bytes: await fs.readFile(file) });
    }
  }
  await collectNotices('THIRD_PARTY_NOTICES.md');
  await collectNotices('licenses');
  if (
    typeof parent.name !== 'string' ||
    typeof parent.version !== 'string' ||
    !parent.version.trim()
  ) {
    throw new Error('Package name and version are required');
  }
  const entries = await fs.readdir(path.join(root, 'npm'), {
    withFileTypes: true
  });
  const platforms = [];
  const prepared = [];
  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    if (entry.isSymbolicLink())
      throw new Error('Platform package symlinks are unsupported');
    if (!entry.isDirectory()) continue;
    const relative = `npm/${entry.name}/package.json`;
    const filename = path.join(root, relative);
    const manifest = JSON.parse(await fs.readFile(filename, 'utf8'));
    if (manifest.name !== `${parent.name}-${entry.name}`)
      throw new Error(
        `Platform package name does not match parent: ${relative}`
      );
    platforms.push({
      name: manifest.name as string,
      platform: entry.name,
      path: relative,
      previousVersion: manifest.version as string,
      version: parent.version as string
    });
    manifest.version = parent.version;
    if (license !== undefined && Array.isArray(manifest.files)) {
      manifest.files = [...new Set([...manifest.files, 'LICENSE'])];
    }
    if (notices.length && Array.isArray(manifest.files)) {
      manifest.files = [
        ...new Set([
          ...manifest.files,
          ...notices.map((notice) => notice.relative.split(path.sep)[0])
        ])
      ];
    }
    if (entry.name.startsWith('linux-')) {
      if (entry.name.endsWith('-gnu')) manifest.libc = ['glibc'];
      else if (entry.name.endsWith('-musl')) manifest.libc = ['musl'];
      else
        throw new Error(
          `Linux platform ABI must specify gnu or musl: ${relative}`
        );
    }
    prepared.push({ filename, manifest });
  }
  if (!platforms.length) throw new Error('No platform packages found');
  parent.optionalDependencies = {
    ...parent.optionalDependencies,
    ...Object.fromEntries(platforms.map((pkg) => [pkg.name, parent.version]))
  };
  if (notices.length && Array.isArray(parent.files)) {
    parent.files = [
      ...new Set([
        ...parent.files,
        ...notices.map((notice) => notice.relative.split(path.sep)[0])
      ])
    ];
  }
  if (write) {
    for (const item of prepared) {
      await fs.writeFile(
        item.filename,
        `${JSON.stringify(item.manifest, null, 2)}\n`
      );
      if (license !== undefined) {
        await fs.writeFile(
          path.join(path.dirname(item.filename), 'LICENSE'),
          license
        );
      }
      for (const notice of notices) {
        const destination = path.join(
          path.dirname(item.filename),
          notice.relative
        );
        await fs.mkdir(path.dirname(destination), { recursive: true });
        await fs.writeFile(destination, notice.bytes);
      }
    }
    await fs.writeFile(parentPath, `${JSON.stringify(parent, null, 2)}\n`);
  }
  return {
    package: parent.name as string,
    version: parent.version as string,
    applied: write,
    published: false,
    licensePrepared: license !== undefined,
    noticeFilesPrepared: notices.map((notice) =>
      notice.relative.replaceAll(path.sep, '/')
    ),
    platforms,
    optionalDependencies: parent.optionalDependencies as Record<string, string>,
    scope:
      'Manifest preparation only; binary presence, pack content and installation still require validation.'
  };
}
