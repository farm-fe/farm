import fs from "node:fs/promises";
import { chromium } from "playwright-chromium";
import assert from "node:assert/strict";
const files = [
  "remote-a/Button.js",
  "remote-b/Button.js",
  "remote-a/button.css",
];
const original = new Map(
  await Promise.all(
    files.map(async (file) => [
      file,
      await fs.readFile(new URL(file, import.meta.url), "utf8"),
    ]),
  ),
);
const write = (file, text) =>
  fs.writeFile(new URL(file, import.meta.url), text);
const evidence = {
  format: process.env.MF_FORMAT || "global",
  passed: false,
  checks: [],
  errors: [],
  consoleErrors: [],
  requestFailures: [],
  sockets: [],
  states: [],
  statuses: [],
};
let browser;
try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  page.on("pageerror", (e) => {
    evidence.errors.push(e.message);
    console.error(e.stack);
  });
  let navigations = 0;
  let phase = "hot-update";
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) navigations++;
  });
  page.on("websocket", (socket) => evidence.sockets.push(socket.url()));
  page.on("requestfailed", (request) =>
    evidence.requestFailures.push({
      url: request.url(),
      error: request.failure()?.errorText,
      phase,
    }),
  );
  page.on("console", (m) => {
    if (m.type() === "error") evidence.consoleErrors.push(m.text());
    console.log(m.type(), m.text());
  });
  await page.addInitScript(() => {
    window.hotStatuses = [];
    window.addEventListener("farm:mf:hot-status", (e) => {
      window.hotStatuses.push(e.detail);
      if (e.detail.state === "reload-required")
        sessionStorage.setItem("mf-hot-fallback", JSON.stringify(e.detail));
    });
  });
  await page.goto("http://127.0.0.1:4100");
  await page.waitForFunction(() => window.hmrProbe, {}, { timeout: 20000 });
  const token = await page.evaluate(() => window.hmrProbe.token);
  for (const id of ["host-count", "react-a", "react-b"])
    await page.locator("#" + id).click();
  const stable = async () => {
    assert.equal(navigations, 1);
    evidence.states.push(
      await page.evaluate(() => ({
        token: window.hmrProbe.token,
        host: document.querySelector("#host-count").textContent,
        a: document.querySelector("#react-a").textContent,
        b: document.querySelector("#react-b").textContent,
        shared: window.hmrProbe.shared(),
      })),
    );
    assert.equal(await page.evaluate(() => window.hmrProbe.token), token);
    assert.equal(await page.locator("#host-count").textContent(), "host:1");
    assert.equal(await page.evaluate(() => window.hmrProbe.shared()), true);
    assert.equal(await page.evaluate(() => window.hmrProbe.repeat()), true);
  };
  const aHot = original
    .get("remote-a/Button.js")
    .replace('"remote-a:"', '"remote-a-hot:"');
  await write("remote-a/Button.js", aHot);
  await page.waitForFunction(
    () => document.querySelector("#react-a")?.textContent === "remote-a-hot:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  assert.equal(await page.locator("#react-b").textContent(), "remote-b:1");
  evidence.checks.push(
    "static remote JS updates without navigation; host and both remote hook states preserved",
  );
  await write(
    "remote-b/Button.js",
    original
      .get("remote-b/Button.js")
      .replace('"remote-b:"', '"remote-b-hot:"'),
  );
  await page.waitForFunction(
    () => document.querySelector("#react-b")?.textContent === "remote-b-hot:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  evidence.checks.push("dynamic second remote updates independently");
  await write("remote-a/button.css", ".remote-a { color: rgb(1, 123, 45); }\n");
  await page.waitForFunction(
    () =>
      getComputedStyle(document.querySelector("#css-a")).color ===
      "rgb(1, 123, 45)",
    {},
    { timeout: 15000 },
  );
  await stable();
  assert.equal(
    await page.locator("#css-b").evaluate((e) => getComputedStyle(e).color),
    "rgb(153, 51, 17)",
  );
  evidence.checks.push(
    "remote CSS updates in place without changing peer styles or application state",
  );
  await Promise.all([
    write(
      "remote-a/Button.js",
      aHot.replace("remote-a-hot:", "remote-a-concurrent:"),
    ),
    write(
      "remote-b/Button.js",
      original
        .get("remote-b/Button.js")
        .replace('"remote-b:"', '"remote-b-concurrent:"'),
    ),
  ]);
  await page.waitForFunction(
    () =>
      document.querySelector("#react-a")?.textContent ===
        "remote-a-concurrent:1" &&
      document.querySelector("#react-b")?.textContent ===
        "remote-b-concurrent:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  evidence.checks.push(
    "concurrent remote updates preserve independent module systems and shared identity",
  );
  await write("remote-a/Button.js", aHot + "\nexport const broken = ;\n");
  await page.waitForFunction(
    () => window.hotStatuses.some((s) => s.state === "compile-error"),
    {},
    { timeout: 15000 },
  );
  await stable();
  await write(
    "remote-a/Button.js",
    aHot.replace("remote-a-hot:", "remote-a-fixed:"),
  );
  await page.waitForFunction(
    () =>
      document.querySelector("#react-a")?.textContent === "remote-a-fixed:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  evidence.checks.push(
    "compile failure retains application; corrected source recovers without reload",
  );
  await write(
    "remote-a/Button.js",
    aHot + '\nthrow new Error("mf-hmr-runtime-probe");\n',
  );
  await page.waitForFunction(
    () => window.hotStatuses.some((s) => s.state === "rolled-back"),
    {},
    { timeout: 15000 },
  );
  await stable();
  await write(
    "remote-a/Button.js",
    aHot.replace("remote-a-hot:", "remote-a-recovered:"),
  );
  await page.waitForFunction(
    () =>
      document.querySelector("#react-a")?.textContent ===
      "remote-a-recovered:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  evidence.checks.push(
    "execution failure rolls back module cache and exports, subsequent update recovers",
  );
  await write("remote-a/Button.js", aHot + "\nexport const newExport = 1;\n");
  await page.waitForFunction(
    () =>
      window.hotStatuses.some(
        (s) =>
          s.state === "reload-required" && s.reason.includes("Export shape"),
      ),
    {},
    { timeout: 15000 },
  );
  await stable();
  await write(
    "remote-a/Button.js",
    aHot.replace("remote-a-hot:", "remote-a-compatible:"),
  );
  await page.waitForFunction(
    () =>
      document.querySelector("#react-a")?.textContent ===
      "remote-a-compatible:1",
    {},
    { timeout: 15000 },
  );
  await stable();
  evidence.checks.push(
    "incompatible export shape reports reload-required, preserves old application and accepts compatible repair",
  );
  evidence.navigationCountBeforeDisconnect = navigations;
  evidence.styleIds = await page
    .locator("style[data-farm-id]")
    .evaluateAll((nodes) => nodes.map((n) => n.getAttribute("data-farm-id")));
  assert.equal(evidence.styleIds.length, 2);
  assert.equal(new Set(evidence.styleIds).size, 2);
  evidence.statuses = await page.evaluate(() => window.hotStatuses);
  assert.deepEqual(evidence.requestFailures, []);
  phase = "disconnect";
  await page.request.get("http://127.0.0.1:4101/__mf_hmr_test_disconnect");
  await page.waitForFunction(
    (old) => window.hmrProbe?.token && window.hmrProbe.token !== old,
    token,
    { timeout: 20000 },
  );
  const recoveredToken = await page.evaluate(() => window.hmrProbe.token);
  assert.equal(await page.locator("#host-count").textContent(), "host:0");
  evidence.disconnectFallback = await page.evaluate(() =>
    JSON.parse(sessionStorage.getItem("mf-hot-fallback")),
  );
  assert.match(evidence.disconnectFallback.reason, /connection lost/);
  await write(
    "remote-a/Button.js",
    aHot.replace("remote-a-hot:", "remote-a-reconnected:"),
  );
  await page.waitForFunction(
    () =>
      document.querySelector("#react-a")?.textContent ===
      "remote-a-reconnected:0",
    {},
    { timeout: 15000 },
  );
  assert.equal(
    await page.evaluate(() => window.hmrProbe.token),
    recoveredToken,
  );
  assert.equal(await page.evaluate(() => window.hmrProbe.shared()), true);
  evidence.checks.push(
    "disconnect explicitly reloads to resynchronize; subsequent JS hot update works without further navigation",
  );
  evidence.navigationCountAfterDisconnect = navigations;
  assert.equal(navigations, 2);
  assert.deepEqual(evidence.errors, []);
  assert.deepEqual(evidence.consoleErrors, []);
  // The intentional reconnect reload aborts its own liveness ping. No other
  // URL/error or failure before disconnect is accepted.
  evidence.expectedDisconnectAborts=evidence.requestFailures.filter(f=>f.phase==="disconnect"&&f.url==="http://127.0.0.1:4101/deploy/a/__hmr"&&f.error==="net::ERR_ABORTED");
  assert.deepEqual(evidence.requestFailures,evidence.expectedDisconnectAborts);
  evidence.passed = true;
} catch (error) {
  evidence.error = error.stack;
  process.exitCode = 1;
} finally {
  for (const [file, text] of original) await write(file, text);
  await browser?.close();
  await fs.writeFile(
    new URL(`evidence-hmr-${evidence.format}.json`, import.meta.url),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify(evidence, null, 2));
}
