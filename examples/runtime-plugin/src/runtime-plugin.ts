import { Plugin } from '@farmfe/runtime';

export default <Plugin>{
  name: 'runtime-plugin-example',
  loadResource: (resource, targetEnv) => {
    console.log('loadResource', resource, targetEnv);
    return import('./replaced.js').then(() => {
      return {
        success: true
      };
    });
  }
};
