import { chromium } from "playwright-chromium";
import { access } from "node:fs/promises";
import { runSuite } from "./e2e-suite.mjs";
export default async function (ctx) {
  // Fail before builds if the browser required by the child harnesses is absent.
  await access(chromium.executablePath());
  await runSuite(ctx);
}
