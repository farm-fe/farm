import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { readFileSync } from 'node:fs';
interface FileNode {
  isLeaf: boolean;
  name: string;
  children: FileNode[];
}

export function generateFileTree(files: string[]): FileNode[] {
  const fileTree: FileNode[] = [];

  for (const file of files) {
    const parts = file.split('/');
    let currentNode = fileTree;

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const existingNode = currentNode.find((node) => node.name === part);

      if (existingNode) {
        currentNode = existingNode.children;
      } else {
        const newNode: FileNode = {
          isLeaf: i === parts.length - 1,
          name: part,
          children: []
        };
        currentNode.push(newNode);
        currentNode = newNode.children;
      }
    }
  }

  return fileTree;
}

export function buildFileTreeHtml(node: FileNode[]): string {
  let html = '';

  for (const fileNode of node) {
    const { isLeaf, name, children } = fileNode;
    const indent = isLeaf ? '- ' : '|---- ';
    const path = name.replace(/ /g, '%20');
    html += `<div>${indent}<a href="${path}">${name}</a></div>`;
    if (!isLeaf) {
      html += buildFileTreeHtml(children).replace(
        /^/gm,
        '&nbsp;&nbsp;&nbsp;&nbsp;'
      );
    }
  }

  return html;
}

export function generateFileTreeHtml(node: FileNode[]): string {
  return `
      <!DOCTYPE html>
      <html lang="">
        <head>
          <meta charset="utf-8">
          <meta http-equiv="X-UA-Compatible" content="IE=edge">
          <meta name="viewport" content="width=device-width,initial-scale=1.0,user-scalable=no">
          <title>Out Files</title>
        </head>
        <body>
          <!-- file tree-->
         <div>${buildFileTreeHtml(node)}</div>
        </body>
      </html>
      `;
}

export function getDependenciesRecursive(config: any) {
  const content = readFileSync(config.resolveConfigPath, 'utf-8');
  const dependencyRegex = /import\s.*?from\s['"](.+?)['"]/g;
  const requireRegex = /require\s*\(\s*['"](.+?)['"]\s*\)/g;
  const allDependencies = [];
  const dependencies = [];

  let match;
  while ((match = dependencyRegex.exec(content)) !== null) {
    dependencies.push(match[1]);
  }

  while ((match = requireRegex.exec(content)) !== null) {
    dependencies.push(match[1]);
  }

  for (const dependency of dependencies) {
    const dependencyPath = path.resolve(
      path.dirname(config.filePath),
      dependency
    );
    // 检查依赖项是否在项目内部，而不是在node_modules中
    if (!dependencyPath.includes('node_modules')) {
      allDependencies.push(dependencyPath);
      getDependenciesRecursive(dependencyPath);
    }
  }

  return allDependencies;
}

export function isInternalDependency(dependencyPath: string) {
  const projectRoot = path.resolve(__dirname);
  return dependencyPath.startsWith(projectRoot);
}

export async function importFresh(modulePath: string) {
  const cacheBustingModulePath = `${modulePath}?update=${Date.now()}`;
  if (process.platform === 'win32') {
    return (await import(pathToFileURL(cacheBustingModulePath).toString()))
      .default;
  } else {
    return (await import(cacheBustingModulePath)).default;
  }
}
