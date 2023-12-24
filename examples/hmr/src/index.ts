// self accept without reload the page
import { data } from "./data";
import { AcceptDepsString } from "./accept-deps-string";
import { AcceptDepsArray } from "./accept-deps-array";
import { SelfAcceptedEmpty } from "./self-accepted-empty";
import { SelfAcceptedFn } from "./self-accepted-fn";
import { InvalidateParent } from "./invalidate-parent";
import { Dispose } from "./dispose";

import './prune';

function render() {
  const root = document.getElementById("root");
  // remove all children of root
  root!.innerHTML = "";

  const renderData = data();
  const div = document.createElement("div");
  div.id = "root-comp";
  div.innerText = renderData;
  root?.appendChild(div);

  const comps = [
    AcceptDepsArray(),
    AcceptDepsString(),
    SelfAcceptedEmpty(),
    SelfAcceptedFn(),
    InvalidateParent(),
    Dispose()
  ];

  comps.forEach(comp => {
    root?.appendChild(comp.render());
  });
}

render();

if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept();

  render();
}