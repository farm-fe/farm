// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore ignore type check
import isPlainObject from 'is-plain-object';
import deepmerge from 'deepmerge';

export default function merge(
  target: Record<string, unknown>,
  ...sources: Record<string, unknown>[]
): Record<string, unknown> {
  target = { ...target };

  for (const source of sources) {
    target = deepmerge(target, source, {
      arrayMerge: (target, source, options) => {
        const destination = target.slice();

        source.forEach((item, index) => {
          if (typeof destination[index] === 'undefined') {
            destination[index] = options.cloneUnlessOtherwiseSpecified(
              item,
              options
            );
          } else if (options.isMergeableObject(item)) {
            destination[index] = deepmerge(target[index], item, options);
          } else if (target.indexOf(item) === -1) {
            destination.push(item);
          }
        });
        return destination;
      },
      isMergeableObject: isPlainObject
    });
  }

  return target;
}
