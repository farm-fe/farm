export function isArray(val: unknown): val is unknown[] {
  return Array.isArray(val);
}

export function isRegExp(reg: unknown): reg is RegExp {
  return Object.prototype.toString.call(reg) === '[object RegExp]';
}

export function isString(val: unknown) {
  return typeof val === 'string';
}

export function resolveIncludes(
  includes: (string | RegExp) | (string | RegExp)[]
) {
  if (isArray(includes)) {
    return includes.map((one) => resolveInclude(one));
  }
  return [resolveInclude(includes)];
}

export function resolveExcludes(
  excludes: (string | RegExp) | (string | RegExp)[]
) {
  if (isArray(excludes)) {
    return excludes.map((one) => resolveExclude(one));
  }
  return [resolveExclude(excludes)];
}

function resolveExclude(value: string | RegExp): RegExp {
  if (isString(value)) {
    return new RegExp(value);
  }
  if (isRegExp(value)) {
    return value;
  }
}

function resolveInclude(value: string | RegExp): string {
  if (isString(value)) {
    return value as string;
  }
  if (isRegExp(value)) {
    return value.toString().slice(1, -1);
  }
}
