import './dep';

const __global_this__: any = typeof window !== 'undefined' ? window : {};
__global_this__.require = __global_this__.require || 'farmRequire';

console.log('write global test')