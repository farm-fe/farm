import __glob__3_3 from "../dir/foo.js";
import __glob__3_2 from "../dir/foo.cjs";
import __glob__3_1 from "../dir/bar.mjs";
import __glob__3_0 from "../dir/bar.js";
import { setup as __glob__1_3 } from "../dir/foo.js";
import { setup as __glob__1_2 } from "../dir/foo.cjs";
import { setup as __glob__1_1 } from "../dir/bar.mjs";
import { setup as __glob__1_0 } from "../dir/bar.js";
const modules = {
    "/dir/bar.js": ()=>import("../dir/bar.js").then((m)=>m.setup),
    "/dir/bar.mjs": ()=>import("../dir/bar.mjs").then((m)=>m.setup),
    "/dir/foo.cjs": ()=>import("../dir/foo.cjs").then((m)=>m.setup),
    "/dir/foo.js": ()=>import("../dir/foo.js").then((m)=>m.setup)
};
const modulesEager = {
    "/dir/bar.js": __glob__1_0,
    "/dir/bar.mjs": __glob__1_1,
    "/dir/foo.cjs": __glob__1_2,
    "/dir/foo.js": __glob__1_3
};
const modulesDefault = {
    "/dir/bar.js": ()=>import("../dir/bar.js").then((m)=>m.default),
    "/dir/bar.mjs": ()=>import("../dir/bar.mjs").then((m)=>m.default),
    "/dir/foo.cjs": ()=>import("../dir/foo.cjs").then((m)=>m.default),
    "/dir/foo.js": ()=>import("../dir/foo.js").then((m)=>m.default)
};
const modulesDefaultEager = {
    "/dir/bar.js": __glob__3_0,
    "/dir/bar.mjs": __glob__3_1,
    "/dir/foo.cjs": __glob__3_2,
    "/dir/foo.js": __glob__3_3
};
