const fs = require('fs');
const { join } = require('path');

const root = process.cwd();

const node_modules_dir = join(root, 'node_modules');

if (!fs.existsSync(node_modules_dir)) {
  fs.mkdirSync(node_modules_dir, { recursive: true });
}

['vendor1', 'vendor2'].forEach((name) => {
  fs.writeFileSync(
    join(node_modules_dir, name + '.js'),
    'module.exports = ' + JSON.stringify(name) + ';'
  );
});
