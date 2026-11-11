import { jsx } from "react/jsx-runtime";
import react from "react";
export { react, jsx };
import "./button.css";
let count = 0;
export function mount(parent) {
  const button = document.createElement("button");
  button.className = "remote-a";
  button.id = "button-a";
  button.textContent = "Remote a";
  button.onclick = async () => {
    try {
      const { label } = await import("./lazy.js");
      button.textContent = label + ":" + ++count;
    } catch (error) {
      window.lastLazyError = error.message;
    }
  };
  parent.append(button);
}

export function Counter() {
  const [count, setCount] = react.useState(0);
  return jsx("button", {
    id: "react-a",
    onClick: () => setCount(count + 1),
    children: "remote-a:" + count,
  });
}
