import { defineConfig } from '@farmfe/core';
import path from 'path';
import { cwd } from 'process';
import Pages from 'vite-plugin-pages';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  vitePlugins:[
   Pages({
      resolver:'react',
      moduleId:"~react-page",
      onRoutesGenerated(routes) {
          return convertToRootPaths(routes,cwd());
      },
    }),
  ]
});

function convertToRootPaths(tree, basePath = '.') {
  return tree.map(node => {
    // 创建新的节点对象，以避免修改原始对象
    const newNode = { ...node };
    // 如果节点有 element 字段，则修改它
    if (newNode.element) {
      newNode.element = `${basePath}${newNode.element}`;
    }
    // 如果节点有 children 字段且它是数组，则递归调用此函数
    if (Array.isArray(newNode.children)) {
      newNode.children = convertToRootPaths(newNode.children, basePath);
    }
    return newNode;
  });
}
