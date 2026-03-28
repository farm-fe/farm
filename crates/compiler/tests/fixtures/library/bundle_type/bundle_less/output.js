//index-22b7ddb3.js:
 const DEFAULT_PREFIX = 'Hello, ';
export { DEFAULT_PREFIX as DEFAULT_PREFIX };


//index-75838095.js:
 function formatName(name) {
    return name.trim().toLowerCase();
}
export { formatName as formatName };


//index.js:
 import { formatName } from "utils.ts";
import { DEFAULT_PREFIX } from "constants.ts";
function createMessage(name) {
    return DEFAULT_PREFIX + formatName(name);
}
export { DEFAULT_PREFIX as DEFAULT_PREFIX, createMessage as createMessage, formatName as formatName };
