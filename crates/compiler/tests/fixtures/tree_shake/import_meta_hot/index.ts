import {a, invalidate } from "./dep"

console.log(a);

const id = "InvalidateParent";

export function InvalidateParent() {
  return {
    render: () => {
      const renderData = invalidate();

      const div = document.createElement("div", {});
      div.id = id;
      div.innerText = renderData;
      div.className = "box";
      return div;
    }
  }
}

if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept();
  const div = document.getElementById(id);

  if (div) {
    const comp = InvalidateParent().render();
    console.log(div, comp);
    div.replaceWith(comp);
  }
}
