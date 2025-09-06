import fs from 'node:fs/promises';
import path from 'node:path';
import { execSync } from 'child_process';

/**
 * Farm plugin prepublish command, publish all packages under npm directory
 */
export async function prepublish(): Promise<void> {
  const npmDir = path.join(process.cwd(), 'npm');
  const packages = await fs.readdir(npmDir);
  const currentPkgJson = JSON.parse(
    await fs.readFile(path.join(process.cwd(), 'package.json'), 'utf-8')
  );
  const currentPackageVersion = currentPkgJson.version;
  const packageNames = [];

  for (const pkg of packages) {
    const pkgJsonPath = path.join(npmDir, pkg, 'package.json');
    const pkgJson = JSON.parse(await fs.readFile(pkgJsonPath, 'utf-8'));
    const pkgName = pkgJson.name;
    pkgJson.version = currentPackageVersion;

    await fs.writeFile(pkgJsonPath, JSON.stringify(pkgJson, null, 2));

    // execute npm publish under the pkg directory
    execSync(`npm publish`, {
      cwd: path.join(npmDir, pkg),
      stdio: 'inherit'
    });

    packageNames.push(pkgName);
    console.log(`Published ${pkgName}@${currentPackageVersion}`);
  }

  // set packageNames as optionalDependencies in current package.json
  currentPkgJson.optionalDependencies = {
    ...currentPkgJson.optionalDependencies,
    ...packageNames.reduce((acc, name) => {
      acc[name] = currentPackageVersion;
      return acc;
    }, {})
  };
  // write current package.json
  await fs.writeFile(
    path.join(process.cwd(), 'package.json'),
    JSON.stringify(currentPkgJson, null, 2)
  );
}
