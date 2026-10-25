# Selected homepage icons

These facades import the original published icon implementations without
visiting their complete root barrels. The matching declarations retain the
packages' public component, ref and props types. No SVG code is copied.

The leaf paths were verified against the lockfile's `@tabler/icons-react`
3.19.0 and `lucide-react` 0.394.0. They are package internals rather than stable
public subpath APIs. When updating either dependency, check every selected leaf
against its public export, run strict TypeScript checks, and compare the rendered
icons, homepage interactions and styles in both languages and themes.

Keep this optimization scoped to the three homepage consumers. Do not alias a
whole package to this incomplete selection: other consumers may need additional
exports.
