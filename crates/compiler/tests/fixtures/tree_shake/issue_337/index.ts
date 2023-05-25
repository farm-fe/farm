import cloneBuffer from "./_cloneBuffer";
import resolve from './resolve-uri';

console.log(cloneBuffer(Buffer.from("test")))
console.log(resolve("test"))