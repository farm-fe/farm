import { execSync } from 'child_process';

const result = execSync(
  'cargo metadata --format-version 1 --no-deps --manifest-path Cargo.toml'
);
const data = JSON.parse(result.toString());

const crates = data.packages.map((pkg) => {
  const name = pkg.name;
  const version = pkg.version;
  return { name, version };
});

const matrixSettings = crates
  .map((crate) => {
    const os = ['windows-latest', 'macos-latest', 'ubuntu-latest'];

    const settings = os.map((os) => {
      return {
        os,
        crate: crate.name
      };
    });

    return settings;
  })
  .flat();

for (const setting of matrixSettings) {
  console.log(`- crate: ${setting.os}
  os: ${setting.crate}`);
}
