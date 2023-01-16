# Farm

Super fast web build engine written in rust, support compiling JS/TS/JSX/TSX, css, html natively, support both js and rust plugin to customize compilation process, yet another performant alternative besides webpack/vite.

Note:

- See [RFC-001](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation) for design motivation and principle.
- **This project is still under development. Contributions are welcome**.
- A executable react demo is available under `examples/react`; see [RoadMap](https://github.com/farm-fe/farm/issues/7) for our goals.

This project is built on SWC Project, using swc for html/css/js/tsx/ts/jsx parsing, transforming, optimizing and codegen.
