function readPackage(pkg, context) {
  // Override the manifest of foo@1.x after downloading it from the registry
  if (pkg.name === 'vue-template-compiler' && pkg.version.match(/^2\.\d\.\d/)) {
    pkg.peerDependencies = {
      vue: pkg.version
    };
    context.log(
      `${pkg.name}@${pkg.version} => vue@${pkg.version} in peerDependencies`
    );
  }

  return pkg;
}

module.exports = {
  hooks: {
    readPackage
  }
};
