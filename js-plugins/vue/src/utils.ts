import path from "path";
import crypto from "crypto";
import { outputData } from "./farm-vue-types.js";
export function warn({ id, message }: outputData) {
  console.warn(`[${id}:warn]:"${message}"`);
}

export function error({ id, message }: outputData) {
  console.error(`[${id}-(error)]:"${message}"`);
}

export function parsePath(resolvedPath: string) {
  const { dir, base } = path.parse(resolvedPath);
  const [filename, query] = base.split("?");
  const queryObj: Record<string, string> = {};
  if (query) {
    query.split("&").forEach((keyValue) => {
      const [key, value] = keyValue.split("=");
      queryObj[key] = value;
    });
  }
  return {
    filename,
    filePath: path.join(dir, filename),
    query: queryObj,
  };
}

export function getHash(text: string, start: number = 0, end: number = 8) {
  return crypto
    .createHash("sha256")
    .update(text)
    .digest("hex")
    .substring(start, end)
    .toLocaleLowerCase();
}
