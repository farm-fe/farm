# @farmfe/plugin-tailwindcss

## 2.0.0

### Minor Changes

- b343109: Support loading `tailwindcss.config.js` in both CommonJS and ESM formats, and apply configured color theme extensions during utility generation.

### Patch Changes

- Pin the Rust SDK dependency family for rebuilding against core 3.0.0 and ABI 0.5.2 with nightly-2026-04-10. Native platform artifacts must be rebuilt and validated together with the matching core before publication.
- Keep Worker-specific CommonJS conversion on a separate module graph so dependencies shared with browser chunks retain their original AST and resolution edges.

  Restore dynamic entry registration when a module is reused from persistent cache so cached builds continue to emit Worker scripts.

  Cache and restore every file emitted by a module, replacing repeated output names instead of retaining only the last file. This preserves SDK declaration dependencies on warm builds.

  Invalidate the previous persistent-cache format once after upgrading so incomplete single-file metadata is rebuilt; existing cache directories are preserved.

  Release graph and resource locks before synchronous update callbacks and update_finished hooks, allowing Worker plugins and queued updates to access generated resources without deadlocking.

  Include generated CSS in the development CSS-to-script cache key so watched inputs that change frozen stylesheet output do not reuse stale style injection code.

  Keep development resource requests waiting through asynchronous regeneration, queued updates and updates started by completion listeners, preventing premature synchronous access to native resource locks.

  Notify update completion only after all update_finished hooks finish, so resource readers and subsequent updates cannot race with plugin finalization.

  Preserve source export names in development: whole-graph export mangling is limited to production because changed-source imports must continue to resolve unchanged modules during HMR. Other minification settings are unaffected.

  Restore the documented DSV object-row contract, load CSV/TSV files without an additional loader, and report malformed rows as compilation errors instead of panicking. Consumers of earlier beta field-array output must replace numeric row indices with header keys; field values remain strings.

  Complete the default Tailwind 4.1.12 color palette so utilities resolve every standard color family and the 950 shades, including emerald colors used by the Commerce Lab fixture.

- b343109: Load JSON-serializable Tailwind configuration from `tailwind.config.js` in the Rust TailwindCSS plugin.

## 2.0.0-beta.1

### Minor Changes

- b343109: Support loading `tailwindcss.config.js` in both CommonJS and ESM formats, and apply configured color theme extensions during utility generation.

### Patch Changes

- b343109: Load JSON-serializable Tailwind configuration from `tailwind.config.js` in the Rust TailwindCSS plugin.
