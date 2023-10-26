const modules = {
    "./another/zoo.js": ()=>import("./another/zoo.js"),
    "./dir/bar.js": ()=>import("./dir/bar.js"),
    "./dir/foo.js": ()=>import("./dir/foo.js")
};
