const c = 3;

const aValue = 'a';
var a = aValue;
var d;

console.log(a);
d = 'd';

const b = 'b';

const e = d;

window.d = e;

function AAA() {
  console.log('b');
}

AAA();

{
  let c = 1000;

  console.log(c);
}

export default function () {
  return b;
}
