import { loadCompData } from "./components/comp1.js";

// export function renderDynamicEntry2() {
const root = document.querySelector("#root");
const div = document.createElement("div");
div.innerHTML = `<div>
<div>dynamic entry2: ${loadCompData()}</div>
</div>`;
root.appendChild(div);
// }
