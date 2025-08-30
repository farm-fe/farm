import * as dep from './dep';

function re() {
  console.log("re.index");
}

export { re as h };
export { r } from './dep2';

console.log(dep.e.value, re);