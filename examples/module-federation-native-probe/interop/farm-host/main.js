import { createRoot } from "react-dom/client";
import react from "react";
import { jsx } from "react/jsx-runtime";
import {
  react as remoteReact,
  jsx as remoteJsx,
  mount,
  Counter,
} from "rsp/Button";
mount(document.querySelector("#app"));
const root = document.createElement("section");
document.querySelector("#app").append(root);
createRoot(root).render(jsx(Counter, {}));
window.interop = {
  ready: true,
  react: react === remoteReact,
  jsx: jsx === remoteJsx,
  element: jsx("span", {}).type,
  version: react.version,
};
