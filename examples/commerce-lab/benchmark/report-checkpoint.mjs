import fs from 'node:fs/promises';
import { randomUUID } from 'node:crypto';

export async function writeJsonCheckpoint(target, report, io = fs) {
  const temporary = `${target}.${process.pid}-${randomUUID()}.tmp`;
  const handle = await io.open(temporary, 'wx');
  try {
    await handle.writeFile(JSON.stringify(report, null, 2));
    await handle.sync();
    await handle.close();
    await io.rename(temporary, target);
  } catch (error) {
    await handle.close().catch(() => {});
    await io.unlink(temporary).catch(() => {});
    throw error;
  }
}
