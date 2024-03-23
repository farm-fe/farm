function isObject(value: any): boolean {
  return value !== null && typeof value === 'object';
}

type T = Record<string, any>;

function isValueSameDeep(target: any, source: any): boolean {
  if (target === source) {
    return true;
  }

  if (!isObject(target) || !isObject(source)) {
    return false;
  }

  if (Object.keys(target).length !== Object.keys(source).length) {
    return false;
  }

  for (const key in source) {
    if (!isValueSameDeep(target[key], source[key])) {
      return false;
    }
  }

  return true;
}

export default function merge(target: T, ...sources: Partial<T>[]): T {
  target = { ...target };

  if (!isObject(target)) {
    return target;
  }

  for (const source of sources) {
    if (!isObject(source)) {
      continue;
    }

    for (const key in source) {
      const targetValue = target[key];
      const sourceValue = source[key];

      if (Array.isArray(targetValue) && Array.isArray(sourceValue)) {
        // remove duplicates
        const value = [...targetValue];
        for (const item of sourceValue) {
          if (value.some((v) => isValueSameDeep(v, item))) {
            continue;
          }
          value.push(item);
        }
        target[key] = value;
      } else if (isObject(targetValue) && isObject(sourceValue)) {
        target[key] = merge(targetValue, sourceValue);
      } else if (sourceValue !== undefined) {
        target[key] = sourceValue;
      }
    }
  }

  return target;
}
