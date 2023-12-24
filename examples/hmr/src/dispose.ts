const id = "Dispose";

export function createChild() {
  const child = document.createElement("div", {});
  child.innerText = id + new Date();
  child.className = "box";
  return child;
}


export function Dispose() {
  return {
    render: () => {
      const div = document.createElement("div", {});
      div.id = id;
      const child = createChild();
      div.appendChild(child);
      return div;
    }
  }
}

if (import.meta.hot) {
  // self accept without reload the page
  import.meta.hot.accept(mod => {
    const div = document.getElementById(id);
    div?.appendChild(mod.createChild());
  });
  import.meta.hot.dispose(() => {
    // remove all children of the div
    const div = document.getElementById(id);
    
    if (div) {
      while (div.firstChild) {
        console.log('dispose', div.firstChild);
        div.removeChild(div.firstChild);
      }
    }
  });
}