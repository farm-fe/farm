# @farmfe/plugin-replace-dirname

Native Rust replacement of module path expressions in Farm script modules. Replacements use the module's resolved source file and directory; review generated Node output when moving or distributing a build.

```ts
import replaceDirname from '@farmfe/plugin-replace-dirname';

const plugins = [replaceDirname()];
```

The optional `include` and `exclude` arrays are path filter patterns; they are not project root directory settings. Consult the shipped `IOptions` declaration for the option shape. The helper returns a Rust plugin descriptor and does not run a JavaScript source transform.

Use Core and native plugin binaries from a matching verified distribution. Farm Docs uses this capability as part of its Node build tooling; its final isolated package validation is separate from a workspace build. See LICENSE.
