import * as index from ".";

function re() {
  var re = "internal re";
  console.log("re.dep2", re, index.h);
}

export { re as r };
