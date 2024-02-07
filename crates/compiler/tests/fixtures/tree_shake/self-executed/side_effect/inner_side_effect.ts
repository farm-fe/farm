let foo = 0;

function setFoo() {
  let foo = 0;
  foo++;
}
function getFoo() {
  return foo;
}

let v = setFoo();

console.log(getFoo());

export default {}