let foo = 0;

function setFoo() {
  foo += 1;
}
function getFoo() {
  return foo;
}

function Bar() {
  console.log('Bar');
}

Bar.prototype.foo = setFoo();

console.log(getFoo());

export default {};
