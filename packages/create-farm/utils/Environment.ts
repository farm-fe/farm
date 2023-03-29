import chalk from 'chalk';
/**
 * judgement node version
 *
 */

export function judgeNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split('.')[0], 10);
  const minimumMajorVersion = 16;

  if (requiredMajorVersion < minimumMajorVersion) {
    console.log(
      chalk.yellow(`create-farm unsupported Node.js v${currentVersion}.`)
    );
    console.log(
      chalk.yellow(`Please use Node.js v${minimumMajorVersion} or higher.`)
    );
    process.exit(1);
  }
}
