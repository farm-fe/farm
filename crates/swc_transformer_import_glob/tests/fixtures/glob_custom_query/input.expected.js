const modules = {
    "./dir/bar.js": ()=>import("./dir/bar.js?foo=bar&bar=true"),
    "./dir/foo.js": ()=>import("./dir/foo.js?foo=bar&bar=true")
};
