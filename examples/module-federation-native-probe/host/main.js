import { createRoot } from "react-dom/client";
import { jsx } from "react/jsx-runtime";
import react from "react";
import "./static-consumer.js";
if (location.search.includes("subpath")) await import("./subpath.js");
const hostSystemBeforeRemotes = globalThis.farm_mf_host.m;
const [a, b] = await Promise.all([
  import("remote/Button"),
  import("other/Button"),
]);
a.mount(document.querySelector("#app"));
b.mount(document.querySelector("#app"));
const reactRoot = document.createElement("section");
document.querySelector("#app").append(reactRoot);
createRoot(reactRoot).render(
  jsx(react.Fragment, {
    children: [jsx(a.Counter, {}, "a"), jsx(b.Counter, {}, "b")],
  }),
);
window.probe = {
  ready: true,
  cyclicRemote: async () => {
    const [a, again] = await Promise.all([
      import("remote/Link"),
      import("remote/Link"),
    ]);
    const b = await a.loadPeer();
    const returned = await b.loadPeer();
    return {
      names: [a.name, b.name, returned.name],
      same: a === returned,
      repeated: a === again,
    };
  },
  jsxIdentity: jsx === a.jsx && jsx === b.jsx,
  jsxType: jsx("span", {}).type,
  sharedIdentity: react === a.react && react === b.react,
  reactVersion: react.version,
  reactElement: react.createElement("div").type,
  staticIdentity: globalThis.nativeStaticMount === a.mount,
  hostUnchanged: hostSystemBeforeRemotes === globalThis.farm_mf_host.m,
  repeat: async () => (await import("remote/Button")) === a,
  missingExpose: () => import("remote/Unknown"),
  missingEntry: () => import("missing/Button"),
};
