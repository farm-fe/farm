import { compData } from "./accept-deps-data"

const id = "SelfAcceptedFn";

export function SelfAcceptedFn() {
  return {
    render: () => {
      const renderData = compData(id);

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
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    console.log('hot self accept', mod, div)
    const comp = mod[id]().render();
    console.log(div, comp);
    div?.replaceWith(comp);
  });
}