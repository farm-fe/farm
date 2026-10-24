import { createHash, randomUUID } from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import { OutputDirectoryLock } from '@farmfe/core';

type Phase = 'building' | 'backup-intent' | 'install-intent' | 'complete';
interface Journal {
  schemaVersion: 1;
  token: string;
  root: string;
  output: string;
  staging: string;
  phase: Phase;
  hadOutput?: boolean;
  oldReport?: string | null;
  candidateReport?: string;
  completedReport?: string | null;
  completedOutput?: boolean;
}
interface Operations {
  validateOutput(root: string, output: string): Promise<boolean>;
  assertRealDirectories(root: string, output: string): Promise<void>;
  renameDirectory(source: string, destination: string): Promise<void>;
}
interface Options extends Operations {
  root: string;
  output: string;
  signal?: AbortSignal;
}
function cancelled(signal?: AbortSignal): void {
  if (signal?.aborted)
    throw Object.assign(new Error('[DOCS_CANCELLED] build cancelled'), {
      cause: signal.reason
    });
}
function identity(file: string): string {
  return process.platform === 'win32' ? file.toLowerCase() : file;
}
function digest(bytes: Uint8Array | string): string {
  return createHash('sha256').update(bytes).digest('hex');
}
async function stat(file: string) {
  return fs.lstat(file).catch((error) => {
    if (error.code === 'ENOENT') return undefined;
    throw error;
  });
}
async function regularJson(file: string): Promise<unknown> {
  const info = await stat(file);
  if (!info || !info.isFile() || info.isSymbolicLink() || info.size > 65536)
    throw new Error(
      `[DOCS_TRANSACTION_RECORD] expected a small regular record: ${file}`
    );
  try {
    return JSON.parse(await fs.readFile(file, 'utf8'));
  } catch (error) {
    throw new Error(`[DOCS_TRANSACTION_RECORD] invalid record: ${file}`, {
      cause: error
    });
  }
}
async function report(
  root: string,
  directory: string,
  operations: Operations
): Promise<string | null | undefined> {
  if (!(await operations.validateOutput(root, directory))) return undefined;
  const file = path.join(directory, 'docs-build-report.json');
  const info = await stat(file);
  if (!info) return null; // validateOutput only permits an empty directory without a report.
  if (!info.isFile() || info.isSymbolicLink())
    throw new Error(`[DOCS_TRANSACTION_RECORD] invalid output report: ${file}`);
  return digest(await fs.readFile(file));
}
function validHash(value: unknown): value is string {
  return typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
}
function validRecord(
  value: unknown,
  root: string,
  output: string
): value is Journal {
  if (!value || typeof value !== 'object') return false;
  const item = value as Journal;
  if (
    item.schemaVersion !== 1 ||
    !/^[a-f0-9-]{36}$/.test(item.token ?? '') ||
    item.root !== root ||
    item.output !== output ||
    typeof item.staging !== 'string' ||
    path.dirname(item.staging) !== path.join(root, '.farm-docs') ||
    !/^build-[A-Za-z0-9_-]+$/.test(path.basename(item.staging)) ||
    !['building', 'backup-intent', 'install-intent', 'complete'].includes(
      item.phase
    )
  )
    return false;
  if (
    item.phase !== 'building' &&
    (typeof item.hadOutput !== 'boolean' ||
      !validHash(item.candidateReport) ||
      (item.hadOutput && item.oldReport !== null && !validHash(item.oldReport)))
  )
    return false;
  if (
    item.phase === 'complete' &&
    (typeof item.completedOutput !== 'boolean' ||
      (item.hadOutput && !item.completedOutput) ||
      (item.completedOutput &&
        item.completedReport !== item.oldReport &&
        item.completedReport !== item.candidateReport) ||
      (item.completedOutput &&
        item.completedReport !== null &&
        !validHash(item.completedReport)))
  )
    return false;
  return true;
}
async function save(file: string, journal: Journal): Promise<void> {
  const temporary = `${file}.${journal.token}.${randomUUID()}.tmp`;
  const handle = await fs.open(temporary, 'wx');
  try {
    try {
      await handle.writeFile(JSON.stringify(journal, null, 2));
      await handle.sync();
    } finally {
      await handle.close();
    }
    await fs.rename(temporary, file);
  } finally {
    await fs.rm(temporary, { force: true });
  }
}
async function marker(journal: Journal): Promise<void> {
  const value = (await regularJson(
    path.join(journal.staging, 'owner.json')
  )) as Partial<Journal>;
  if (
    !value ||
    typeof value !== 'object' ||
    value.token !== journal.token ||
    value.root !== journal.root ||
    value.output !== journal.output ||
    value.staging !== journal.staging
  )
    throw new Error(
      `[DOCS_TRANSACTION_OWNERSHIP] staging marker mismatch: ${journal.staging}`
    );
}
async function cleanupOwnership(journal: Journal, file: string): Promise<void> {
  const owner = await stat(path.join(journal.staging, 'owner.json'));
  if (owner) await marker(journal);
  if (owner && !(await stat(`${file}.cleanup`))) return;
  // Recursive deletion may already have removed the in-directory marker.
  // A separate complete-phase receipt must still authenticate this exact path.
  const receipt = await regularJson(`${file}.cleanup`);
  if (
    journal.phase !== 'complete' ||
    !validRecord(receipt, journal.root, journal.output) ||
    receipt.phase !== 'complete' ||
    receipt.token !== journal.token ||
    receipt.staging !== journal.staging ||
    receipt.completedOutput !== journal.completedOutput ||
    receipt.completedReport !== journal.completedReport
  )
    throw new Error(
      `[DOCS_TRANSACTION_OWNERSHIP] cleanup receipt mismatch: ${file}`
    );
}
async function removeStaging(
  journal: Journal,
  file: string,
  operations: Operations
): Promise<void> {
  await operations.assertRealDirectories(journal.root, journal.staging);
  if (await stat(journal.staging)) {
    await cleanupOwnership(journal, file);
    await fs.rm(journal.staging, { recursive: true, force: true });
  }
}
async function recover(
  file: string,
  root: string,
  output: string,
  operations: Operations
): Promise<void> {
  if (!(await stat(file))) {
    if (!(await stat(`${file}.cleanup`))) return;
    const receipt = await regularJson(`${file}.cleanup`);
    if (!validRecord(receipt, root, output) || receipt.phase !== 'complete')
      throw new Error(
        `[DOCS_TRANSACTION_RECORD] invalid orphan cleanup receipt: ${file}`
      );
    await save(file, receipt);
  }
  const value = await regularJson(file);
  if (!validRecord(value, root, output))
    throw new Error(
      `[DOCS_TRANSACTION_RECORD] incompatible transaction retained at ${file}`
    );
  const journal = value;
  await operations.assertRealDirectories(root, journal.staging);
  const stageExists = Boolean(await stat(journal.staging));
  if (stageExists || journal.phase === 'complete')
    await cleanupOwnership(journal, file);
  else
    throw new Error(
      `[DOCS_OUTPUT_RECOVERY] staging missing; transaction retained at ${file}`
    );
  const previous = path.join(journal.staging, 'previous');
  const outputReport = await report(root, output, operations);
  // Complete cleanup may already have removed part of the old backup. Its
  // public replacement and separate receipt must authenticate further cleanup.
  const cleanupAuthorized =
    journal.phase === 'complete' && Boolean(await stat(`${file}.cleanup`));
  await operations.assertRealDirectories(root, previous);
  const previousExists = Boolean(await stat(previous));
  const previousReport =
    stageExists && !cleanupAuthorized
      ? await report(root, previous, operations)
      : undefined;
  if (journal.phase === 'building') {
    if (previousReport !== undefined)
      throw new Error(
        `[DOCS_OUTPUT_RECOVERY] unexpected backup retained at ${previous}`
      );
    // No output move was authorized in this phase.
    journal.completedOutput = outputReport !== undefined;
    journal.completedReport = outputReport;
    journal.hadOutput = outputReport !== undefined;
    journal.oldReport = outputReport;
    journal.candidateReport = digest('no-candidate');
  } else if (journal.phase === 'complete') {
    if (
      journal.completedOutput !== (outputReport !== undefined) ||
      (journal.completedOutput && outputReport !== journal.completedReport)
    )
      throw new Error(
        `[DOCS_OUTPUT_RECOVERY] completed output changed; transaction retained at ${file}`
      );
    if (
      previousExists &&
      (!journal.completedOutput ||
        !journal.hadOutput ||
        (!cleanupAuthorized && previousReport !== journal.oldReport))
    )
      throw new Error(
        `[DOCS_OUTPUT_RECOVERY] unverified backup retained at ${previous}`
      );
  } else if (outputReport === undefined) {
    if (journal.hadOutput) {
      if (previousReport === undefined || previousReport !== journal.oldReport)
        throw new Error(
          `[DOCS_OUTPUT_RECOVERY] verified backup missing; transaction retained at ${file}`
        );
      await operations.renameDirectory(previous, output);
      journal.completedOutput = true;
      journal.completedReport = journal.oldReport;
    } else {
      if (previousReport !== undefined)
        throw new Error(
          `[DOCS_OUTPUT_RECOVERY] unexpected backup retained at ${previous}`
        );
      journal.completedOutput = false;
    }
  } else if (
    journal.phase === 'install-intent' &&
    outputReport === journal.candidateReport
  ) {
    // Installation completed before the process could persist its completion.
    if (previousReport !== undefined && previousReport !== journal.oldReport)
      throw new Error(
        `[DOCS_OUTPUT_RECOVERY] backup changed; retained at ${previous}`
      );
    journal.completedOutput = true;
    journal.completedReport = outputReport;
  } else if (
    journal.hadOutput &&
    outputReport === journal.oldReport &&
    previousReport === undefined
  ) {
    journal.completedOutput = true;
    journal.completedReport = outputReport;
  } else
    throw new Error(
      `[DOCS_OUTPUT_RECOVERY] output conflicts with transaction; retained at ${file}`
    );
  // A crash after staging cleanup can now be recognized without its marker.
  journal.phase = 'complete';
  await save(file, journal);
  await save(`${file}.cleanup`, journal);
  await removeStaging(journal, file, operations);
  await fs.rm(file);
  await fs.rm(`${file}.cleanup`);
  console.warn(`[DOCS_OUTPUT_RECOVERED] ${output}`);
}

export interface OutputTransaction {
  staging: string;
  destination: string;
  commit(): Promise<void>;
}

/** File orchestration only. Content, routes and resource relationships stay native. */
export async function withOutputTransaction<T>(
  options: Options,
  run: (transaction: OutputTransaction) => Promise<T>
): Promise<T> {
  const { root, output, signal } = options;
  cancelled(signal);
  const metadata = path.join(root, '.farm-docs-locks');
  await options.assertRealDirectories(root, metadata);
  await fs.mkdir(metadata, { recursive: true });
  const key = digest(identity(output));
  const record = path.join(metadata, `${key}.json`);
  // Site-wide ownership also prevents ancestor/descendant outputs racing.
  const lock = new OutputDirectoryLock(path.join(metadata, 'site.lock'));
  try {
    // Recover all site transactions before creating any output. Otherwise a
    // new nested output could occupy an ancestor whose backup needs restoring.
    const records = new Set<string>();
    for (const name of await fs.readdir(metadata)) {
      const match = name.match(/^([a-f0-9]{64}\.json)(?:\.cleanup)?$/);
      if (match) records.add(match[1]);
      else if (name.endsWith('.json'))
        throw new Error(
          `[DOCS_TRANSACTION_RECORD] unknown record retained at ${path.join(metadata, name)}`
        );
    }
    for (const name of [...records].sort()) {
      const file = path.join(metadata, name);
      const value = (await regularJson(
        (await stat(file)) ? file : `${file}.cleanup`
      )) as Partial<Journal>;
      const target =
        value && typeof value === 'object' ? value.output : undefined;
      const relative =
        typeof target === 'string' ? path.relative(root, target) : '';
      if (
        typeof target !== 'string' ||
        path.resolve(target) !== target ||
        !relative ||
        relative === '..' ||
        relative.startsWith(`..${path.sep}`) ||
        path.isAbsolute(relative) ||
        ['.git', 'node_modules', '.farm-docs', '.farm-docs-locks'].includes(
          relative.split(path.sep)[0].toLowerCase()
        ) ||
        `${digest(identity(target))}.json` !== name
      )
        throw new Error(
          `[DOCS_TRANSACTION_RECORD] invalid output identity retained at ${file}`
        );
      await recover(file, root, target, options);
    }
    cancelled(signal);
    await options.validateOutput(root, output);
    await options.assertRealDirectories(root, path.join(root, '.farm-docs'));
    await fs.mkdir(path.join(root, '.farm-docs'), { recursive: true });
    const staging = await fs.mkdtemp(path.join(root, '.farm-docs/build-'));
    const journal: Journal = {
      schemaVersion: 1,
      token: randomUUID(),
      root,
      output,
      staging,
      phase: 'building'
    };
    const destination = path.join(staging, 'public');
    let backup = false,
      installed = false,
      failure: unknown,
      failed = false;
    let outcome: { ok: true; value: T } | { ok: false; error: unknown };
    try {
      await fs.writeFile(
        path.join(staging, 'owner.json'),
        JSON.stringify(journal),
        { flag: 'wx' }
      );
      await save(record, journal);
      const result = await run({
        staging,
        destination,
        commit: async () => {
          cancelled(signal);
          const candidate = await report(root, destination, options);
          if (!candidate)
            throw new Error(
              '[DOCS_TRANSACTION_RECORD] candidate report missing'
            );
          const old = await report(root, output, options);
          journal.hadOutput = old !== undefined;
          journal.oldReport = old;
          journal.candidateReport = candidate;
          journal.phase = 'backup-intent';
          await save(record, journal);
          cancelled(signal);
          if (journal.hadOutput) {
            await options.renameDirectory(
              output,
              path.join(staging, 'previous')
            );
            backup = true;
          }
          await fs.mkdir(path.dirname(output), { recursive: true });
          cancelled(signal);
          journal.phase = 'install-intent';
          await save(record, journal);
          cancelled(signal);
          // From this rename onward installation/recovery must finish despite abort.
          await options.renameDirectory(destination, output);
          installed = true;
        }
      });
      outcome = { ok: true, value: result };
    } catch (error) {
      failed = true;
      failure = error;
      outcome = { ok: false, error };
    }
    {
      if (!installed && backup) {
        try {
          await options.renameDirectory(path.join(staging, 'previous'), output);
        } catch (error) {
          throw new AggregateError(
            failed ? [failure, error] : [error],
            `[DOCS_OUTPUT_RECOVERY] unable to restore output; previous build retained at ${path.join(staging, 'previous')}; journal ${record}`
          );
        }
      }
      try {
        // Do not erase a recoverable record until the public state is known.
        const completed = await report(root, output, options);
        journal.completedOutput = completed !== undefined;
        journal.completedReport = completed;
        // Failed builds may never reach commit; capture enough completion data
        // to distinguish staging cleanup from an interrupted output move.
        if (journal.phase === 'building') {
          journal.hadOutput = completed !== undefined;
          journal.oldReport = completed;
          journal.candidateReport = digest('no-candidate');
        }
        journal.phase = 'complete';
        await save(record, journal);
        await save(`${record}.cleanup`, journal);
        await removeStaging(journal, record, options);
        await fs.rm(record);
        await fs.rm(`${record}.cleanup`);
      } catch (error) {
        throw new AggregateError(
          failed ? [failure, error] : [error],
          `[DOCS_OUTPUT_CLEANUP] ${installed ? 'new output committed' : 'output not replaced'}; staging retained at ${staging}; journal ${record}`
        );
      }
    }
    if (!outcome.ok) throw outcome.error;
    return outcome.value;
  } finally {
    lock.release();
  }
}
