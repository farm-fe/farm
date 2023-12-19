import { compData } from "./comp-data"

export function Comp() {
  return {
    render: () => {
      const renderData = compData();

      const div = document.createElement("div", {});
      div.innerText = renderData;
      return div;
    }
  }
}