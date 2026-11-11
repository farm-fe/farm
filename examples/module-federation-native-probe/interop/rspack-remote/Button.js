import react from "react";
import { jsx } from "react/jsx-runtime";
import "./style.css";
export { react, jsx };
export function mount(parent) {
  const b = document.createElement("button");
  b.id = "rsp-button";
  b.className = "rsp";
  b.textContent = "Rspack";
  b.onclick = async () => {
    b.textContent = (await import("./lazy.js")).label;
  };
  parent.append(b);
}

export function Counter() {
  const [count, setCount] = react.useState(0);
  return jsx("button", {
    id: "interop-counter",
    onClick: () => setCount(count + 1),
    children: "interop:" + count,
  });
}
