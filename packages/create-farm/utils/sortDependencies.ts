export default function sortDependencies(packageJson: Record<string, any>) {
  const sorted: Record<string, any> = {};

  const depTypes = [
    'dependencies',
    'devDependencies',
    'peerDependencies',
    'optionalDependencies'
  ];

  for (const depType of depTypes) {
    if (packageJson[depType]) {
      sorted[depType] = {};

      Object.keys(packageJson[depType])
        .sort()
        .forEach((name) => {
          sorted[depType][name] = packageJson[depType][name];
        });
    }
  }

  return {
    ...packageJson,
    ...sorted
  };
}
