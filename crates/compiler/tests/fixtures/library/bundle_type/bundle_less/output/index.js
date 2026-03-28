import { formatName } from "lib/utils.ts";
import { DEFAULT_PREFIX } from "lib/constants.ts";
function createMessage(name) {
    return DEFAULT_PREFIX + formatName(name);
}
export { DEFAULT_PREFIX as DEFAULT_PREFIX, createMessage as createMessage, formatName as formatName };
