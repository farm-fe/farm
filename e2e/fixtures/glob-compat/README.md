# Glob build/dev acceptance

This fixture tests real Farm production builds, browser imports, literal pattern
arrays, negative ordering, braces, lazy/eager and named/default imports, query
key stability, base/root/alias keys, exhaustive discovery, HMR add/delete, and
warm persistent-cache membership after file additions/removals.
It uses no real network service or application data.

Build the current Core native and JavaScript outputs first. With an already
installed `playwright-core` and Chromium, run from the repository root:

```sh
FARM_GLOB_PLAYWRIGHT=/absolute/path/to/playwright-core \
FARM_GLOB_CHROMIUM=/absolute/path/to/chromium \
node scripts/test-glob-compat.mjs
```

Use the equivalent environment-variable syntax in PowerShell. The script does
not download a browser. Port 4367 must be free. It creates and removes only its
known `src/files/added.js` fixture, refuses a preexisting file, and closes its
browser and server. Generated `.output` resources are ignored.

Production and development receive separate configuration objects because
resolution normalizes configuration in place. Development explicitly selects
its HMR client port and waits for watcher readiness before changing files.
