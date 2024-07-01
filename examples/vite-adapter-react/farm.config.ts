import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';
import react from '@farmfe/plugin-react';
import farmPluginHtmlTemplate from '@jstors/farm-plugin-html-template';
import path from 'path';

export default defineConfig({
  plugins: [
    '@farmfe/plugin-react',
    farmPluginHtmlTemplate({
      template: path.resolve(__dirname, './index.html'),
      data: { title: 'Farm React App' }
    })
    // [
    //   '@jstors/farm-plugin-html-template',
    //   {
    //     template: path.resolve(__dirname, './index.html'),
    //     data: { title: 'Farm React App' }
    //   }
    // ]
  ],
  vitePlugins: [
    Pages({
      resolver: 'react'
    })
  ]
});
