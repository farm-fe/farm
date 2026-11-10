import * as ns from "./value.js";
import { "s-k" as named, "" as empty, "雪" as unicode, increment } from "./value.js";
import { "re-export" as forwarded, "name-space" as namespace } from "./bridge.js";
export function values() { return [ns["s-k"], named, empty, unicode, forwarded, namespace["s-k"]]; }
export { increment, named as "public-name" };
export async function boundaryValues() {
  const boundary = await import("./boundary.js");
  return [boundary["1-name"], boundary["123"], boundary["other-name"], boundary["boundary-namespace"]["s-k"], boundary[""], boundary.readExisting()];
}
