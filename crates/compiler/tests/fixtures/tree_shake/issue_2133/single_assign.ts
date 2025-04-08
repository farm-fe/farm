// @ts-ignore
import { target1, target as target2, target3 } from "fake-module";

// reference
var _a1, _a2;
(_a2 = (_a1 = target2).__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS) != null ? _a2 : _a1.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS = [];

// no reference
var _b1, _b2;
(_b2 = (_b1 = target1).__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS) != null ? _b2 : _b1.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS = [];

target3.namespace = 234;

var singleAssign = new Proxy(target2.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
  get(target22, prop, receiver) {
    return Reflect.get(target22, prop, receiver);
  }
});

console.log({ singleAssign })