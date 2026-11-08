import { "s-k" as existing } from "./value.js";
export function readExisting() { return existing; }
export * from "./value.js";
export { "1-name", "123", "s-k" as "other-name" } from "./value.js";
export * as "boundary-namespace" from "./value.js";
