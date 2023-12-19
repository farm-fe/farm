// self accept without reload the page
import { data } from "./data";

function render() {
  const root = document.getElementById("root");
  const renderData = data();

  const div = document.createElement("div");
  div.id = "root-comp";
  div.innerText = renderData;
  root?.appendChild(div);
}

render();