import react from "react";
import { jsx } from "react/jsx-runtime";
import { createRoot } from "react-dom/client";
import { flushSync } from "react-dom";
import { Counter as StaticCounterA } from "remote/Button";
import { bridge } from "farm:module-federation";
const a = await import("remote/Button"),
  b = await import("other/Button");
const root = createRoot(document.querySelector("#app"));
function RemoteA() {
  return StaticCounterA();
}
function RemoteB() {
  return b.Counter();
}
function Host() {
  const [n, set] = react.useState(0);
  return jsx("section", {
    children: [
      jsx(
        "button",
        { id: "host-count", onClick: () => set(n + 1), children: "host:" + n },
        "host",
      ),
      jsx(RemoteA, {}, "a"),
      jsx(RemoteB, {}, "b"),
      jsx(
        "div",
        { className: "remote-a", id: "css-a", children: "css-a" },
        "css-a",
      ),
      jsx(
        "div",
        { className: "remote-b", id: "css-b", children: "css-b" },
        "css-b",
      ),
    ],
  });
}
function render() {
  flushSync(() => root.render(jsx(Host, {})));
}
bridge.acceptRemote("remote/Button", render);
bridge.acceptRemote("other/Button", render);
render();
window.hmrProbe = {
  token: crypto.randomUUID(),
  shared: () => react === a.react && react === b.react,
  repeat: async () => (await import("remote/Button")) === a,
};
