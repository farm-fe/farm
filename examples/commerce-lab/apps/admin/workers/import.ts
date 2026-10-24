import Papa from 'papaparse';
import { validateImport } from '../../../packages/domain/index.mjs';

self.onmessage = ({ data }) => {
  try {
    const parsed = Papa.parse(data.csv, { header: true, skipEmptyLines: true });
    if (parsed.errors.length)
      throw new Error(parsed.errors.map((error) => error.message).join('; '));
    const rows = parsed.data as Array<Record<string, string>>;
    self.postMessage({
      requestId: data.requestId,
      rows,
      ...validateImport(rows, data.existing),
    });
  } catch (error) {
    self.postMessage({
      requestId: data.requestId,
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
