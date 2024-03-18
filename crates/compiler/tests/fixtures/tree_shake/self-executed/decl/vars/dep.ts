function foo() {
  console.log('hello world');
}

let foo1 = 1,
  foo2 = 2;

foo1 = 2;

export { foo1, foo2 };

var foo3 = foo;
var foo4 = foo;

foo3.create = foo;

export { foo3, foo4 };
