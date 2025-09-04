import e from "node:fs";

export function readFileSync(path) {
  return e.readFileSync(path, "utf-8");
}
