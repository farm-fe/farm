// @ts-nocheck
console.log(import.meta.glob("bar", { exhaustive: true }));
console.log(import.meta.glob("@foo/core", { exhaustive: true }));