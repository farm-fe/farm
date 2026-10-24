import type { DocsFarmConfig } from './index.js';

type Alias = NonNullable<
  NonNullable<NonNullable<DocsFarmConfig['compilation']>['resolve']>['alias']
>;

/** Keep ordered user aliases intact while reserving the framework runtime identities. */
export function withFrameworkAliases(
  user: Alias | undefined,
  framework: Record<string, string>
): Alias {
  return Array.isArray(user)
    ? [
        ...Object.entries(framework).map(([find, replacement]) => ({
          find,
          replacement
        })),
        ...user
      ]
    : { ...user, ...framework };
}
