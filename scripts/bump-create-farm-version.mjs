import fs from 'node:fs';
import path from 'node:path';

const templatesDir = './templates'; 

const dependenciesToUpdate = {
  "@farmfe/cli": "", // TODO dynamic
  "@farmfe/core": "" // TODO dynamic
};

fs.readdir(templatesDir, (err, files) => {
  if (err) {
    return console.error(`can not read template dir: ${err.message}`);
  }

  files.forEach(file => {
    const projectDir = path.join(templatesDir, file);
    const packageJsonPath = path.join(projectDir, 'package.json');

    if (fs.existsSync(packageJsonPath)) {
      fs.readFile(packageJsonPath, 'utf8', (err, data) => {
        if (err) {
          return console.error(`can not read file ${packageJsonPath}: ${err.message}`);
        }

        let packageJson;
        try {
          packageJson = JSON.parse(data);
        } catch (err) {
          return console.error(`resolve JSON file ${packageJsonPath} error: ${err.message}`);
        }

        Object.keys(dependenciesToUpdate).forEach(dep => {
          if (packageJson.dependencies && packageJson.dependencies[dep]) {
            packageJson.dependencies[dep] = dependenciesToUpdate[dep];
          }
          if (packageJson.devDependencies && packageJson.devDependencies[dep]) {
            packageJson.devDependencies[dep] = dependenciesToUpdate[dep];
          }
        });

        fs.writeFile(packageJsonPath, JSON.stringify(packageJson, null, 2), 'utf8', err => {
          if (err) {
            return console.error(`can not write ${packageJsonPath}: ${err.message}`);
          }
          console.log(`success update file ${packageJsonPath}`);
        });
      });
    }
  });
});