import { compData } from "./accept-deps-data"

const id = "AcceptDepsString";

export function AcceptDepsString() {
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
  // accept dependencies
  import.meta.hot.accept('./accept-deps-data', (data) => {
    console.log(data);
    const div = document.getElementById(id);
    const renderData = data.compData(id);
    div!.innerText = renderData;
  });
}