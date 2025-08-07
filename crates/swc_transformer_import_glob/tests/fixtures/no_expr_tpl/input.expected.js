const modulesDefault = {
    "./dir/bar.js": ()=>import("./dir/bar.js").then((m)=>m.default),
    "./dir/foo.js": ()=>import("./dir/foo.js").then((m)=>m.default)
};
