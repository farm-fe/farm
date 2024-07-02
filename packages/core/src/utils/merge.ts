import deepmerge, { Options } from 'deepmerge';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore ignore type check
import { isPlainObject } from 'is-plain-object';
import { isArray } from './share.js';

function isValueSameDeep(target: any, source: any): boolean {
  if (target === source) {
    return true;
  }

  if (!isMergeableObject(target) || !isMergeableObject(source)) {
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

function isMergeableObject(obj: any) {
  return isPlainObject(obj) || Array.isArray(obj);
}

const arrayMerge: Options['arrayMerge'] = (target, source, options) => {
  const destination = target.slice();

  source.forEach((item, index) => {
    if (typeof destination[index] === 'undefined') {
      destination[index] = options.cloneUnlessOtherwiseSpecified(item, options);
    } else if (!destination.find((dest) => isValueSameDeep(dest, item))) {
      destination.push(item);
    }
  });

  return destination.filter((item) => item !== undefined);
};

const options = {
  arrayMerge,
  isMergeableObject
};

export default function merge<T>(target: T, ...sources: Partial<T>[]): T {
  let destination: any = { ...target };

  for (const source of sources) {
    if (!source) continue;

    // should not preserve target and source
    if (isPlainObject(destination) && isPlainObject(source)) {
      for (const key of Object.keys(source)) {
        const sourceValue = (source as any)[key];

        if (sourceValue === undefined) {
          continue;
        } else if (
          isMergeableObject(destination[key]) &&
          isMergeableObject(sourceValue)
        ) {
          destination[key] = deepmerge(destination[key], sourceValue, options);
        } else {
          if (isPlainObject(sourceValue)) {
            destination[key] = deepmerge({}, sourceValue, options);
          } else if (isArray(sourceValue)) {
            destination[key] = deepmerge([], sourceValue, options);
          } else {
            destination[key] = sourceValue;
          }
        }
      }
    } else {
      destination = deepmerge(destination, source, options);
    }
  }

  return destination;
}
