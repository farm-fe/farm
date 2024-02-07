let cache = {};

export function set(key, obj) {
  cache[key] = obj;
}

export function get(key) {
  return cache[key];
}
