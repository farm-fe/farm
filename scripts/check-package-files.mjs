/**
 * CI check: verify that every published package has a `files` field in
 * package.json and that statically-committed files listed in `files` exist on
 * disk.
 *
 * Rules:
 *  - rust-plugins/<name>/package.json
 *      • must define a non-empty `files` field
 *      • must include "index.js" and "index.d.ts"
 *      • every entry in `files` that looks like a source file
 *        (*.js / *.cjs / *.mjs / *.d.ts / *.ts) must exist on disk
 *  - rust-plugins/<name>/npm/<abi>/package.json
 *      • must define a non-empty `files` field
 *      • must include "index.farm"
 *  - js-plugins/<name>/package.json
 *      • must define a non-empty `files` field  (skip private packages)
 *  - packages/<name>/package.json
 *      • must define a non-empty `files` field  (skip private packages)
 */

import { existsSync, readdirSync, statSync, readFileSync } from "node:fs";
import { join, resolve, extname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(fileURLToPath(new URL(".", import.meta.url)), "..");

let errors = 0;

function fail(msg) {
  console.error(`  ✗ ${msg}`);
  errors++;
}

function pass(msg) {
  console.log(`  ✓ ${msg}`);
}

function readJson(filePath) {
  try {
    return JSON.parse(readFileSync(filePath, "utf-8"));
  } catch {
    return null;
  }
}

function isDir(p) {
  try {
    return statSync(p).isDirectory();
  } catch {
    return false;
  }
}

function subdirs(dir) {
  if (!isDir(dir)) return [];
  return readdirSync(dir).filter((f) => isDir(join(dir, f)));
}

/** Returns true when the path looks like a source file that must be committed
 *  to the repository (not a build artifact directory or binary).
 *  Note: path.extname("index.d.ts") returns ".ts", so ".ts" covers ".d.ts" too. */
function isSourceFile(entry) {
  const ext = extname(entry).toLowerCase();
  return [".js", ".cjs", ".mjs", ".ts"].includes(ext);
}

// ---------------------------------------------------------------------------
// 1. Rust plugins – main packages
// ---------------------------------------------------------------------------
console.log("\n── Rust plugins (main packages) ──");

const RUST_PLUGINS_DIR = join(ROOT, "rust-plugins");
const RUST_PLUGIN_REQUIRED_ENTRIES = ["index.js", "index.d.ts"];

for (const pluginName of subdirs(RUST_PLUGINS_DIR)) {
  const pkgDir = join(RUST_PLUGINS_DIR, pluginName);
  const pkgJsonPath = join(pkgDir, "package.json");
  const pkgJson = readJson(pkgJsonPath);
  const label = `rust-plugins/${pluginName}`;

  if (!pkgJson) {
    fail(`${label}: cannot read package.json`);
    continue;
  }

  const files = pkgJson.files;

  if (!files || files.length === 0) {
    fail(`${label}: missing or empty "files" field in package.json`);
    continue;
  }

  let ok = true;

  // Required minimum entries
  for (const req of RUST_PLUGIN_REQUIRED_ENTRIES) {
    if (!files.includes(req)) {
      fail(`${label}: "files" must include "${req}" (got: ${JSON.stringify(files)})`);
      ok = false;
    }
  }

  // Every source-file entry must exist on disk
  for (const entry of files) {
    if (isSourceFile(entry)) {
      if (!existsSync(join(pkgDir, entry))) {
        fail(`${label}: "${entry}" is listed in "files" but does not exist on disk`);
        ok = false;
      }
    }
  }

  if (ok) pass(`${label}: OK`);
}

// ---------------------------------------------------------------------------
// 2. Rust plugins – npm platform sub-packages
// ---------------------------------------------------------------------------
console.log("\n── Rust plugins (npm sub-packages) ──");

for (const pluginName of subdirs(RUST_PLUGINS_DIR)) {
  const npmDir = join(RUST_PLUGINS_DIR, pluginName, "npm");
  if (!isDir(npmDir)) continue;

  for (const abi of subdirs(npmDir)) {
    const subPkgDir = join(npmDir, abi);
    const pkgJsonPath = join(subPkgDir, "package.json");
    const pkgJson = readJson(pkgJsonPath);
    const label = `rust-plugins/${pluginName}/npm/${abi}`;

    if (!pkgJson) {
      fail(`${label}: cannot read package.json`);
      continue;
    }

    const files = pkgJson.files;

    if (!files || files.length === 0) {
      fail(`${label}: missing or empty "files" field in package.json`);
      continue;
    }

    if (!files.includes("index.farm")) {
      fail(`${label}: "files" must include "index.farm" (got: ${JSON.stringify(files)})`);
    } else {
      pass(`${label}: OK`);
    }
  }
}

// ---------------------------------------------------------------------------
// 3. JS plugins
// ---------------------------------------------------------------------------
console.log("\n── JS plugins ──");

const JS_PLUGINS_DIR = join(ROOT, "js-plugins");

for (const pluginName of subdirs(JS_PLUGINS_DIR)) {
  const pkgJsonPath = join(JS_PLUGINS_DIR, pluginName, "package.json");
  const pkgJson = readJson(pkgJsonPath);
  const label = `js-plugins/${pluginName}`;

  if (!pkgJson) {
    fail(`${label}: cannot read package.json`);
    continue;
  }

  if (pkgJson.private) {
    pass(`${label}: private package, skipping`);
    continue;
  }

  const files = pkgJson.files;
  if (!files || files.length === 0) {
    fail(`${label}: missing or empty "files" field in package.json`);
  } else {
    pass(`${label}: OK`);
  }
}

// ---------------------------------------------------------------------------
// 4. packages/*
// ---------------------------------------------------------------------------
console.log("\n── packages/* ──");

const PACKAGES_DIR = join(ROOT, "packages");

for (const pkgName of subdirs(PACKAGES_DIR)) {
  const pkgJsonPath = join(PACKAGES_DIR, pkgName, "package.json");
  const pkgJson = readJson(pkgJsonPath);
  const label = `packages/${pkgName}`;

  if (!pkgJson) {
    fail(`${label}: cannot read package.json`);
    continue;
  }

  if (pkgJson.private) {
    pass(`${label}: private package, skipping`);
    continue;
  }

  const files = pkgJson.files;
  if (!files || files.length === 0) {
    fail(`${label}: missing or empty "files" field in package.json`);
  } else {
    pass(`${label}: OK`);
  }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------
console.log("");
if (errors > 0) {
  console.error(`❌  ${errors} error(s) found. Please fix the issues above.\n`);
  process.exit(1);
} else {
  console.log(`✅  All package files checks passed.\n`);
}
