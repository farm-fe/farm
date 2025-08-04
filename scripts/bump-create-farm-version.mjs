import fs from 'node:fs';
import path from 'node:path';

const templatesDir = path.resolve('crates/create-farm-rs/templates'); 

const packageCorePath = path.resolve('packages/core/package.json');
const packageReactPluginPath = path.resolve('rust-plugins/react/package.json');
const packageCliPath = path.resolve('packages/cli/package.json');
const packageCoreJson = JSON.parse(fs.readFileSync(packageCorePath, 'utf8'));
const packageCliJson = JSON.parse(fs.readFileSync(packageCliPath, 'utf8'));
const packageReactPluginJson = JSON.parse(fs.readFileSync(packageReactPluginPath, 'utf8'));
const dependenciesToUpdate = {
  "farm": `^${packageCoreJson.version}`,
  "@farmfe/plugin-react": `^${packageReactPluginJson.version}`,
};

function updatePackageJsonDependencies(dir) {
  fs.readdir(dir, (err, files) => {
    if (err) {
      return console.error(`cannot read directory ${dir}: ${err.message}`);
    }

    files.forEach(file => {
      const fullPath = path.join(dir, file);
      fs.stat(fullPath, (err, stats) => {
        if (err) {
          return console.error(`cannot get stats of file ${fullPath}: ${err.message}`);
        }

        if (stats.isDirectory()) {
          updatePackageJsonDependencies(fullPath);
        } else if (file === 'package.json') {
          fs.readFile(fullPath, 'utf8', (err, data) => {
            if (err) {
              return console.error(`cannot read file ${fullPath}: ${err.message}`);
            }

            let packageJson;
            try {
              packageJson = JSON.parse(data);
            } catch (err) {
              return console.error(`resolve JSON file ${fullPath} error: ${err.message}`);
            }

            Object.keys(dependenciesToUpdate).forEach(dep => {
              if (packageJson.dependencies && packageJson.dependencies[dep]) {
                packageJson.dependencies[dep] = dependenciesToUpdate[dep];
              }
              if (packageJson.devDependencies && packageJson.devDependencies[dep]) {
                packageJson.devDependencies[dep] = dependenciesToUpdate[dep];
              }
            });

            fs.writeFile(fullPath, JSON.stringify(packageJson, null, 2), 'utf8', err => {
              if (err) {
                return console.error(`cannot write ${fullPath}: ${err.message}`);
              }
              console.log(`successfully updated file ${fullPath}`);
            });
          });
        }
      });
    });
  });
}

updatePackageJsonDependencies(templatesDir);
