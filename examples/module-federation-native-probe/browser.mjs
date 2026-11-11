import http from "node:http";
import path from "node:path";
import fs from "node:fs/promises";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { chromium } from "playwright-chromium";

const base = path.dirname(fileURLToPath(import.meta.url));
const servers = [];
const evidence = {
  format: process.env.MF_FORMAT || "global",
  csp: !!process.env.MF_CSP,
  serve: process.env.MF_SERVE || "static",
  mode: process.env.MF_MODE || "production",
  requests: [],
  checks: [],
  errors: [],
};
const types = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".map": "application/json",
};
let browser;
try {
  if (process.env.MF_SERVE !== "dev")
    for (const [dir, port, prefix] of [
      ["host", 4100, "/"],
      ["remote-a", 4101, "/deploy/a/"],
      ["remote-b", 4102, "/deploy/b/"],
    ]) {
      const root = path.join(base, "dist", dir);
      const server = http.createServer(async (req, res) => {
        const pathname = new URL(req.url, "http://localhost").pathname;
        const relative =
          decodeURIComponent(pathname.slice(prefix.length)) || "index.html";
        const file = path.resolve(root, relative);
        res.setHeader("Access-Control-Allow-Origin", "*");
        res.setHeader("Cache-Control", "no-store");
        if (process.env.MF_CSP)
          res.setHeader(
            "Content-Security-Policy",
            "script-src 'self' http://127.0.0.1:4101 http://127.0.0.1:4102; style-src 'self' http://127.0.0.1:4101 http://127.0.0.1:4102; object-src 'none'; base-uri 'none'",
          );
        if (!pathname.startsWith(prefix) || !file.startsWith(root + path.sep)) {
          res.writeHead(404);
          res.end("Not found");
          return;
        }
        try {
          const content = await fs.readFile(file);
          res.setHeader(
            "Content-Type",
            types[path.extname(file)] || "application/octet-stream",
          );
          res.end(content);
        } catch {
          res.writeHead(404);
          res.end("Not found");
        }
      });
      await new Promise((resolve, reject) => {
        server.once("error", reject);
        server.listen(port, "127.0.0.1", resolve);
      });
      servers.push(server);
    }
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.addInitScript(() => {
    window.__mfScriptEntries = [];
    new MutationObserver((records) => {
      for (const record of records)
        for (const node of record.addedNodes) {
          if (node.nodeName === "SCRIPT" && node.src.includes("remoteEntry.js"))
            window.__mfScriptEntries.push({ src: node.src, type: node.type });
        }
    }).observe(document, { childList: true, subtree: true });
  });
  page.on("response", (r) =>
    evidence.requests.push({ url: r.url(), status: r.status() }),
  );
  page.on("pageerror", (e) => {
    evidence.errors.push(e.message);
    console.error(e.stack);
  });
  const consoleErrors = [];
  const requestFailures = [];
  page.on("requestfailed", (request) =>
    requestFailures.push({
      url: request.url(),
      error: request.failure()?.errorText,
    }),
  );
  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });
  await page.goto("http://127.0.0.1:4100");
  await page.waitForFunction(() => window.probe?.ready, {}, { timeout: 30000 });
  assert.equal(
    await page.locator("#button-a").evaluate((e) => getComputedStyle(e).color),
    "rgb(17, 85, 153)",
  );
  assert.equal(
    await page.locator("#button-b").evaluate((e) => getComputedStyle(e).color),
    "rgb(153, 51, 17)",
  );
  await page.locator("#react-a").waitFor();
  assert.equal(await page.locator("#react-a").textContent(), "remote-a:0");
  await page.locator("#react-a").click();
  await page.waitForFunction(
    () => document.querySelector("#react-a").textContent === "remote-a:1",
  );
  assert.equal(await page.locator("#react-b").textContent(), "remote-b:0");
  await page.locator("#react-b").click();
  await page.waitForFunction(
    () => document.querySelector("#react-b").textContent === "remote-b:1",
  );
  evidence.checks.push(
    "host ReactDOM renders both remote hook components with independent state",
  );
  evidence.checks.push("two remotes and computed CSS");
  await page.locator("#button-a").click();
  await page.waitForFunction(
    () => document.querySelector("#button-a").textContent === "lazy-a:1",
  );
  await page.locator("#button-b").click();
  await page.waitForFunction(
    () => document.querySelector("#button-b").textContent === "lazy-b:1",
  );
  await page.locator("#button-a").click();
  await page.waitForFunction(
    () => document.querySelector("#button-a").textContent === "lazy-a:2",
  );
  assert.equal(await page.locator("#button-b").textContent(), "lazy-b:1");
  evidence.checks.push("independent lazy chunks and module state");
  const identity = await page.evaluate(async () => ({
    repeat: await window.probe.repeat(),
    staticIdentity: window.probe.staticIdentity,
    jsxIdentity: window.probe.jsxIdentity,
    jsxType: window.probe.jsxType,
    sharedIdentity: window.probe.sharedIdentity,
    reactVersion: window.probe.reactVersion,
    reactElement: window.probe.reactElement,
    hostUnchanged: window.probe.hostUnchanged,
    namespaces: ["farm_mf_host", "farm_mf_providerA", "farm_mf_providerB"].map(
      (n) => !!window[n]?.m,
    ),
    distinct:
      window.farm_mf_host?.m !== window.farm_mf_providerA?.m &&
      window.farm_mf_providerA?.m !== window.farm_mf_providerB?.m,
    containers: [
      typeof window.remoteA?.get,
      typeof window.remoteA?.init,
      typeof window.remoteB?.get,
    ],
    factories: await Promise.all([
      window.remoteA.get("./Button"),
      window.remoteA.get("./Button"),
    ]).then(([a, b]) => a() === b()),
  }));
  evidence.identity = identity;
  assert.equal(identity.jsxIdentity, true);
  assert.equal(identity.jsxType, "span");
  assert.equal(identity.sharedIdentity, true);
  assert.equal(identity.reactVersion, "18.3.1");
  assert.equal(identity.reactElement, "div");
  evidence.cyclicRemote = await page.evaluate(() =>
    window.probe.cyclicRemote(),
  );
  assert.deepEqual(evidence.cyclicRemote, {
    names: ["a", "b", "a"],
    same: true,
    repeated: true,
  });
  evidence.checks.push(
    "cyclic lazy remote graph A→B→A and concurrent imports preserve module identity",
  );
  evidence.checks.push("React singleton across static host and both remotes");
  assert.equal(identity.staticIdentity, true);
  assert.equal(identity.hostUnchanged, true);
  assert.equal(identity.repeat, true);
  assert.deepEqual(identity.namespaces, [true, true, true]);
  assert.equal(identity.distinct, true);
  assert.deepEqual(identity.containers, ["function", "function", "function"]);
  assert.equal(identity.factories, true);
  if (evidence.format === "global") {
    const entries = await page.evaluate(() => window.__mfScriptEntries);
    assert.equal(entries.length, 2);
    assert.ok(entries.every((s) => !s.type || s.type === "text/javascript"));
    evidence.classicEntries = entries;
  }
  evidence.identity = identity;
  evidence.checks.push(
    "namespace isolation, get/init, concurrent get and module identity",
  );
  assert.deepEqual(evidence.errors, []);
  assert.deepEqual(consoleErrors, []);
  assert.deepEqual(requestFailures, []);
  for (const [port, prefix] of [
    [4101, "/deploy/a/"],
    [4102, "/deploy/b/"],
  ]) {
    const requests = evidence.requests.filter((r) =>
      r.url.startsWith(`http://127.0.0.1:${port}`),
    );
    assert.ok(requests.some((r) => r.url.endsWith(".css")));
    assert.ok(requests.filter((r) => r.url.endsWith(".js")).length >= 3);
    assert.ok(
      requests.every(
        (r) => r.status === 200 && new URL(r.url).pathname.startsWith(prefix),
      ),
    );
  }
  evidence.checks.push("remote-origin JS/CSS deployment subpaths");
  const failures = await page.evaluate(async () => {
    const capture = async (fn) => {
      try {
        await fn();
        return null;
      } catch (e) {
        return e.message;
      }
    };
    return {
      expose: await capture(window.probe.missingExpose),
      entry: await capture(window.probe.missingEntry),
    };
  });
  assert.match(failures.expose, /remote\/Unknown/);
  assert.match(failures.expose, /Unknown expose/);
  assert.match(failures.entry, /missing\/Button/);
  assert.match(failures.entry, /4101\/missing\/remoteEntry.js/);
  evidence.failures = failures;
  evidence.checks.push("missing expose and entry diagnostics");
  // New contexts avoid a previous successful chunk/CSS load masking failure behavior.
  const lazyFailure = await browser.newPage();
  await lazyFailure.goto("http://127.0.0.1:4100");
  await lazyFailure.waitForFunction(() => window.probe?.ready);
  const lazyUrls = evidence.requests.filter(
    (r) =>
      r.url.includes(":4101/") &&
      /\/lazy[._-]/.test(r.url) &&
      r.url.endsWith(".js"),
  );
  const lazyUrl = lazyUrls.at(-1).url;
  await lazyFailure.route(lazyUrl, (route) => route.abort("failed"));
  await lazyFailure.locator("#button-a").click();
  await lazyFailure.waitForFunction(
    () => window.lastLazyError,
    {},
    { timeout: 15000 },
  );
  evidence.lazyFailure = await lazyFailure.evaluate(() => window.lastLazyError);
  assert.ok(evidence.lazyFailure.length);
  evidence.checks.push("missing lazy chunk rejects");
  await lazyFailure.close();
  const cssFailure = await browser.newPage();
  await cssFailure.route("http://127.0.0.1:4101/**/*.css", (route) =>
    route.abort("failed"),
  );
  const cssErrors = [];
  cssFailure.on("pageerror", (e) => cssErrors.push(e.message));
  const cssErrorEvent = cssFailure.waitForEvent("pageerror", {
    timeout: 15000,
  });
  await cssFailure.goto("http://127.0.0.1:4100");
  await cssErrorEvent;
  assert.ok(
    cssErrors.some((e) => /remote\/Button/.test(e)),
    JSON.stringify(cssErrors),
  );
  evidence.cssFailure = cssErrors;
  evidence.checks.push("missing CSS rejects remote load");
  await cssFailure.close();
  if (evidence.format === "global") {
    for (const incompatible of [false, true]) {
      const isolated = await browser.newPage();
      await isolated.route("http://127.0.0.1:4100/isolated", (r) =>
        r.fulfill({
          status: 200,
          contentType: "text/html",
          body: "<!doctype html><title>shared</title>",
        }),
      );
      await isolated.goto("http://127.0.0.1:4100/isolated");
      await isolated.addScriptTag({
        url: "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
      });
      const shared = await isolated.evaluate(async (incompatible) => {
        const scope = incompatible
          ? {
              react: {
                "17.0.2": {
                  version: "17.0.2",
                  from: "incompatible-provider",
                  loaded: true,
                  lib: () => ({ version: "17.0.2" }),
                  scope: ["default"],
                  shareConfig: { singleton: true },
                },
              },
            }
          : {};
        const initScope = [];
        const first = window.remoteA.init(scope, initScope);
        await window.remoteA.init(scope, initScope);
        const samePromise = first === window.remoteA.init(scope);
        await first;
        let differentScope;
        try {
          await window.remoteA.init({});
        } catch (e) {
          differentScope = e.message;
        }
        try {
          const [a, b] = await Promise.all([
            window.remoteA.get("./Button"),
            window.remoteA.get("./Button"),
          ]);
          return {
            samePromise,
            differentScope,
            sameModule: a() === b(),
            version: a().react.version,
          };
        } catch (e) {
          return { samePromise, differentScope, error: e.message };
        }
      }, incompatible);
      assert.equal(shared.samePromise, true);
      assert.match(shared.differentScope, /Different share scope/);
      if (incompatible) {
        assert.match(shared.error, /17\.0\.2/);
        assert.match(shared.error, /18\.3\.1/);
      } else {
        assert.equal(shared.version, "18.3.1");
        assert.equal(shared.sameModule, true);
      }
      evidence[incompatible ? "strictVersion" : "localFallback"] = shared;
      await isolated.close();
    }
    evidence.checks.push(
      "local fallback, strict version rejection, concurrent init/get and different scope rejection",
    );
    const collision = await browser.newPage();
    await collision.route("http://127.0.0.1:4100/collision", (r) =>
      r.fulfill({
        status: 200,
        contentType: "text/html",
        body: "<!doctype html><title>collision</title>",
      }),
    );
    await collision.goto("http://127.0.0.1:4100/collision");
    await collision.evaluate(() => {
      window.remoteA = { occupied: true };
      window.farm_mf_providerA = { m: { sentinel: true } };
    });
    const errorEvent = collision.waitForEvent("pageerror", { timeout: 10000 });
    await collision.addScriptTag({
      url: "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
    });
    const error = await errorEvent;
    assert.match(error.message, /Duplicate container global/);
    assert.equal(await collision.evaluate(() => window.remoteA.occupied), true);
    assert.equal(
      await collision.evaluate(() => window.farm_mf_providerA.m.sentinel),
      true,
    );
    evidence.globalCollision = error.message;
    evidence.checks.push("occupied container global rejects without overwrite");
    await collision.evaluate(() => {
      delete window.remoteA;
    });
    const namespaceError = collision.waitForEvent("pageerror", {
      timeout: 10000,
    });
    await collision.addScriptTag({
      url: "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
    });
    assert.match((await namespaceError).message, /Duplicate runtime namespace/);
    assert.equal(
      await collision.evaluate(() => window.farm_mf_providerA.m.sentinel),
      true,
    );
    assert.equal(
      await collision.evaluate(() => typeof window.remoteA),
      "undefined",
    );
    evidence.checks.push(
      "occupied runtime namespace rejects before any initialization",
    );
    await collision.close();
  }
  const retryPage = await browser.newPage();
  const retryErrors = [];
  retryPage.on("pageerror", (e) => retryErrors.push(e.message));
  const retryUrl = "http://127.0.0.1:4101/deploy/a/remoteEntry.js";
  await retryPage.route(retryUrl, (route) => route.abort("failed"));
  const rejected = retryPage.waitForEvent("pageerror", { timeout: 15000 });
  await retryPage.goto("http://127.0.0.1:4100");
  assert.match((await rejected).message, /remote\/Button/);
  assert.equal(await retryPage.evaluate(() => !!window.probe?.ready), false);
  await retryPage.unroute(retryUrl);
  const priorErrors = retryErrors.length;
  await retryPage.reload();
  await retryPage.waitForFunction(
    () => window.probe?.ready,
    {},
    { timeout: 15000 },
  );
  await retryPage.locator("#react-a").waitFor();
  assert.equal(retryErrors.length, priorErrors);
  evidence.retry = {
    failedBootstrapDoesNotBecomeReady: true,
    recoveredAfterFullReload: true,
    samePageRetry: "not asserted; recovery uses full document reload",
  };
  evidence.checks.push(
    "failed remote entry aborts bootstrap; restored endpoint recovers on full reload",
  );
  await retryPage.close();
  await page.screenshot({ path: path.join(base, "evidence.png") });
  evidence.consoleErrorsIncludingExpectedFailures = consoleErrors;
  evidence.requestFailuresIncludingExpectedFailures = requestFailures;
  evidence.passed = true;
} catch (error) {
  evidence.passed = false;
  evidence.failure = error.stack;
  process.exitCode = 1;
} finally {
  if (browser) await browser.close();
  await Promise.all(
    servers.map((server) => new Promise((resolve) => server.close(resolve))),
  );
  await fs.writeFile(
    path.join(
      base,
      `evidence-${evidence.format}-${evidence.mode}-${evidence.serve}${evidence.csp ? "-csp" : ""}.json`,
    ),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify(evidence, null, 2));
}
