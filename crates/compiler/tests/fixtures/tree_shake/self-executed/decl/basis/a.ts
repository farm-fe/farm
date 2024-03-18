// vars
var a1 = 11,
  a2 = 22,
  a3 = 33;

console.log(a1);

const c = 3;

const aValue = 'a';
var a = aValue;
console.log(a);

var d;

d = 'd';

const e = d;

// window.d = e;

{
  let c = 1000;

  console.log(c);
}

// function
function AAA() {
  console.log('aaa');
}
function BBB() {
  console.log('bbb');
}

AAA();

// class

class Foo {
  constructor() {
    console.log('foo');
  }
}

class Bar {
  constructor() {
    console.log('bar');
  }
}

new Foo();

export default function () {
  console.log('foo');
}
