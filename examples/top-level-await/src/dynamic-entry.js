import { loadCompData } from "./components/comp1.js";
import dep1 from "./dep1.js";
import async2 from "./async2.js";

const root = document.querySelector("#root");
const div = document.createElement("div");
div.innerHTML = `<div>
dep1: ${dep1}
async2: ${JSON.stringify(async2)}

<div>comp-data: ${loadCompData()}</div>
</div>`;
root.appendChild(div);
