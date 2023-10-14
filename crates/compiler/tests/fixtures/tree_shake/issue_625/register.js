const cache = {};

export function getTickMethod(id) {
  return cache[id];
}

export function registerTickMethod(id, method) {
  cache[id] = method;
}