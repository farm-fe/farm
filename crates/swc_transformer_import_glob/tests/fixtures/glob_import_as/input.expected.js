import __glob__1_1 from "./dir/foo.js?url";
import __glob__1_0 from "./dir/bar.js?url";
const modules = {
    "./dir/bar.js": "export default 'bar';",
    "./dir/foo.js": "export default 'foo';"
};
function loadImageUrls() {
    const images = {
        "./dir/bar.js": __glob__1_0,
        "./dir/foo.js": __glob__1_1
    };
    return images;
}
