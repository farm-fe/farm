import __glob__3_1 from "./dir/foo.js";
import __glob__3_0 from "./dir/bar.js";
import { setup as __glob__1_1 } from "./dir/foo.js";
import { setup as __glob__1_0 } from "./dir/bar.js";
const modules = {
    "./dir/bar.js": ()=>import("./dir/bar.js").then((m)=>m.setup),
    "./dir/foo.js": ()=>import("./dir/foo.js").then((m)=>m.setup)
};
const modulesEager = {
    "./dir/bar.js": __glob__1_0,
    "./dir/foo.js": __glob__1_1
};
const modulesDefault = {
    "./dir/bar.js": ()=>import("./dir/bar.js").then((m)=>m.default),
    "./dir/foo.js": ()=>import("./dir/foo.js").then((m)=>m.default)
};
const modulesDefaultEager = {
    "./dir/bar.js": __glob__3_0,
    "./dir/foo.js": __glob__3_1
};
