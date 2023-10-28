const modules = {
    "./dir/bar.js?foo=bar&bar=true": ()=>import("./dir/bar.js?foo=bar&bar=true"),
    "./dir/foo.js?foo=bar&bar=true": ()=>import("./dir/foo.js?foo=bar&bar=true")
};
