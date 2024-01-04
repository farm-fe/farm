import { compData } from "./accept-deps-data"

const id = "SelfAcceptedEmpty";

export function SelfAcceptedEmpty() {
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
  import.meta.hot.accept();

  const div = document.getElementById(id);

  if (div) {
    const comp = SelfAcceptedEmpty().render();
    console.log(div, comp);
    div.replaceWith(comp);
  }
}