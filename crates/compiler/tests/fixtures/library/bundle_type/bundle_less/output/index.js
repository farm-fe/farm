import { formatName } from "./lib/utils.js";
import { DEFAULT_PREFIX } from "./lib/constants.js";
function createMessage(name) {
    return DEFAULT_PREFIX + formatName(name);
}
export { DEFAULT_PREFIX as DEFAULT_PREFIX, createMessage as createMessage, formatName as formatName };
