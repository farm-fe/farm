// import { readFileSync } from 'node:fs';
import type { JsPlugin } from 'farm';

interface Options {
  /* Your options here */
}

export default function farmPlugin(options: Options): JsPlugin {
  return {
    name: '<FARM-JS-PLUGIN-NPM-NAME>',
    /* Your plugin hooks here: */ 

    // config(config) {
    //   console.log('options', options);
    //   return config;
    // },
    // load: {
    //   filters: {
    //     resolvedPaths: ['.js$']
    //   },
    //   async executor(params) {
    //     const { resolvedPath } = params;
    //     const content = readFileSync(resolvedPath, 'utf-8');
    //     return {
    //       content,
    //       moduleType: 'js'
    //     };
    //   }
    // },
    // transform: {
    //   filters: {
    //     moduleTypes: ['js']
    //   },
    //   async executor(params) {
    //     const { content } = params;
    //     return {
    //       content,
    //       moduleType: 'js'
    //     };
    //   }
    // },
    // finish: {
    //   executor() {}
    // }
  };
}
