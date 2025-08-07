function foo() {
  console.log("foo");
}

function bar() {
  console.log("bar");
}

foo.prototype.runFoo = function () {
  console.log("runFoo");
};

const tempBar = bar;

tempBar.prototype.runFoo = foo.prototype.runFoo;

export { bar };