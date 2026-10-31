// cspell:words postpack
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

export const reviewedCorePrepack =
  'node scripts/prepare-sdk-types.mjs && node scripts/prepare-cjs-types.mjs';
const reviewedScripts = {
  'scripts/prepare-sdk-types.mjs': 'fc6663e1e6d8a2f16b92367dd1b5c61103891b1d9bd7b5d31cc0dbf3edd44174',
  'scripts/prepare-cjs-types.mjs': 'c5620328a1a726566857774e9cf1b2a0f31147805adc57fc9a23942c79ace778'
};

/** Only reviewed, deterministic Core declaration preparation may run during pack. */
export function assertReviewedCandidatePackHooks(directory, manifest) {
  for (const hook of ['prepack', 'postpack', 'prepare']) {
    const command = manifest.scripts?.[hook];
    if (!command) continue;
    if (manifest.name !== '@farmfe/core' || hook !== 'prepack' || command !== reviewedCorePrepack) {
      throw Error(`Review candidate pack hook: ${manifest.name}/${hook}`);
    }
    const root = fs.realpathSync(directory);
    for (const [relative, expected] of Object.entries(reviewedScripts)) {
      const file = path.join(root, relative);
      if (fs.realpathSync(file) !== file || !fs.lstatSync(file).isFile()) {
        throw Error(`Review candidate pack script path: ${relative}`);
      }
      // Git's Windows checkout line endings do not change executable content.
      const source = fs.readFileSync(file, 'utf8').replaceAll('\r\n', '\n');
      const digest = crypto.createHash('sha256').update(source).digest('hex');
      if (digest !== expected) throw Error(`Review changed candidate pack script: ${relative}`);
    }
  }
}
