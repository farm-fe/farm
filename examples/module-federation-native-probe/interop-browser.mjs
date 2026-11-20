import http from "node:http";
import path from "node:path";
import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { chromium } from "playwright-chromium";
const bundler = process.env.MF_BUNDLER || "rspack";
const base = path.dirname(fileURLToPath(import.meta.url)),
  servers = [];
let browser;
const evidence = {
  format: process.env.MF_FORMAT || "global",
  bundler,
  version: bundler === "webpack" ? "5.112.2" : "2.3.0",
  mf: "2.9.2",
  react: "18.3.1",
  cases: [],
  passed: false,
};
try {
  for (const [dir, port, prefix] of [
    ["interop-farm-host", 4200, "/"],
    ["rspack-remote", 4201, "/rspack/"],
    ["rspack-host", 4202, "/"],
    ["remote-a", 4101, "/deploy/a/"],
  ]) {
    const root = path.join(base, "dist", dir);
    const server = http.createServer(async (req, res) => {
      const pathname = new URL(req.url, "http://local").pathname;
      const file = path.resolve(
        root,
        decodeURIComponent(pathname.slice(prefix.length)) || "index.html",
      );
      res.setHeader("Access-Control-Allow-Origin", "*");
      res.setHeader(
        "Content-Security-Policy",
        "script-src 'self' http://127.0.0.1:4201 http://127.0.0.1:4101; style-src 'self' http://127.0.0.1:4201 http://127.0.0.1:4101; object-src 'none'; base-uri 'none'",
      );
      if (!pathname.startsWith(prefix) || !file.startsWith(root + path.sep)) {
        res.statusCode = 404;
        res.end();
        return;
      }
      try {
        const body = await fs.readFile(file);
        res.setHeader(
          "Content-Type",
          file.endsWith(".html")
            ? "text/html"
            : file.endsWith(".css")
              ? "text/css"
              : "text/javascript",
        );
        res.end(body);
      } catch {
        res.statusCode = 404;
        res.end();
      }
    });
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(port, "127.0.0.1", resolve);
    });
    servers.push(server);
  }
  browser = await chromium.launch({ headless: true });
  for (const [direction, port, selector, color, label, remotePort, prefix] of [
    [
      "Farm consumes " + bundler,
      4200,
      "#rsp-button",
      "rgb(12, 34, 56)",
      "rspack-lazy",
      4201,
      "/rspack/",
    ],
    [
      bundler + " consumes Farm",
      4202,
      "#button-a",
      "rgb(17, 85, 153)",
      "lazy-a:1",
      4101,
      "/deploy/a/",
    ],
  ]) {
    const page = await browser.newPage(),
      result = {
        direction,
        errors: [],
        requests: [],
        consoleErrors: [],
        requestFailures: [],
      };
    evidence.cases.push(result);
    page.on("pageerror", (e) => result.errors.push(e.message));
    page.on("console", (m) => {
      if (m.type() === "error") result.consoleErrors.push(m.text());
    });
    page.on("requestfailed", (q) =>
      result.requestFailures.push({
        url: q.url(),
        error: q.failure()?.errorText,
      }),
    );
    page.on("response", (r) =>
      result.requests.push({ url: r.url(), status: r.status() }),
    );
    await page.goto("http://127.0.0.1:" + port);
    await page.waitForFunction(
      () => window.interop?.ready,
      {},
      { timeout: 15000 },
    );
    result.identity = await page.evaluate(() => window.interop);
    assert.deepEqual(result.identity, {
      ready: true,
      react: true,
      jsx: true,
      element: "span",
      version: "18.3.1",
    });
    assert.equal(
      await page.locator(selector).evaluate((el) => getComputedStyle(el).color),
      color,
    );
    await page.locator(selector).click();
    await page.waitForFunction(
      ({ selector, label }) =>
        document.querySelector(selector).textContent === label,
      { selector, label },
    );
    const counter = port === 4200 ? "#interop-counter" : "#react-a";
    await page.locator(counter).waitFor();
    await page.locator(counter).click();
    await page.waitForFunction(
      ({ counter, label }) =>
        document.querySelector(counter).textContent === label,
      { counter, label: port === 4200 ? "interop:1" : "remote-a:1" },
    );
    result.renderedRemoteHookComponent = true;
    assert.deepEqual(result.errors, []);
    assert.deepEqual(result.consoleErrors, []);
    assert.deepEqual(result.requestFailures, []);
    const remoteRequests = result.requests.filter(
      (r) => new URL(r.url).port === String(remotePort),
    );
    assert.ok(remoteRequests.some((r) => r.url.endsWith(".css")));
    assert.ok(remoteRequests.filter((r) => r.url.endsWith(".js")).length >= 3);
    assert.ok(
      remoteRequests.every(
        (r) => r.status === 200 && new URL(r.url).pathname.startsWith(prefix),
      ),
    );
    result.passed = true;
    await page.close();
  }
  evidence.passed = true;
} catch (e) {
  evidence.error = e.stack;
  process.exitCode = 1;
} finally {
  await browser?.close();
  await Promise.all(
    servers.map((s) => new Promise((resolve) => s.close(resolve))),
  );
  await fs.writeFile(
    path.join(
      base,
      "evidence-interop-" +
        (bundler === "webpack" ? "webpack-" : "") +
        evidence.format +
        ".json",
    ),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify(evidence, null, 2));
}
