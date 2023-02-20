import path from 'node:path';
import { copyFiles, TEMPLATES_DIR } from '../utils.js';

const TEMPLATE_REACT = path.join(TEMPLATES_DIR, 'react');

export async function create(): Promise<void> {
  const dest = path.join(process.cwd(), 'farm-react');

  copyFiles(TEMPLATE_REACT, dest);
}
