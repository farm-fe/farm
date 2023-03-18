import { execSync } from 'child_process';
import path from 'path';
import fs from 'fs/promises';

/**
 * Farm plugin prepublish command, publish all packages under npm directory
 */
export async function prepublish(_args: any): Promise<void> {
  const npmDir = path.join(process.cwd(), 'npm');
  const packages = await fs.readdir(npmDir);
  const currentPkgJson = JSON.parse(
    await fs.readFile(path.join(process.cwd(), 'package.json'), 'utf-8')
  );
  const currentPackageVersion = currentPkgJson.version;

  for (const pkg of packages) {
    const pkgJsonPath = path.join(npmDir, pkg, 'package.json');
    const pkgJson = JSON.parse(await fs.readFile(pkgJsonPath, 'utf-8'));
    const pkgName = pkgJson.name;
    pkgJson.version = currentPackageVersion;

    await fs.writeFile(pkgJsonPath, JSON.stringify(pkgJson, null, 2));

    // execute npm publish under the pkg directory
    execSync(`npm publish`, {
      cwd: path.join(npmDir, pkg),
      stdio: 'inherit',
    });
    console.log(`Published ${pkgName}@${currentPackageVersion}`);
  }
}
