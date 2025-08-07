{
  const __global_this_dep__: any = typeof window !== 'undefined' ? window : {};
  __global_this_dep__.require = __global_this_dep__.require || 'farmRequire';
}

const __global_this_dep__: any = {};

if (typeof "production" !== 'undefined') {
  __global_this_dep__.production = 'undefined'
}


for (const key in __global_this_dep__) {
  const element = __global_this_dep__[key];
  __global_this_dep__[key] = element + 1;
}