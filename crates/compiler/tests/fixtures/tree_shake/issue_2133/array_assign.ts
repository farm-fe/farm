// @ts-ignore
import { target1, target as target3, target2 } from "fake-module";

// reference
var _a1;
[_a1] = [target3], _a1.namespace = 234;

// no write operation, should be deleted
var _a2;
[_a2] = [target3];

// no reference
var _b1;
[_b1] = [target2], _b1.namespace = 234;

target1.namespace = 234;

var arrayAssign = new Proxy(target3.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
  get(target22, prop, receiver) {
    return Reflect.get(target22, prop, receiver);
  }
});

console.log({ arrayAssign })