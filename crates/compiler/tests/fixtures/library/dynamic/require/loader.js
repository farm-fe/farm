exports.loadTsSync = () => require("./dep");
exports.loadTs = async () => (await import("./dep")).default;
