import { foo as defaultFoo } from './foo';


function loadFoo(foo = defaultFoo) {
  return {}
}

class LoadFoo {
  constructor(public foo = defaultFoo) {}

  getFoo() {
    return this.foo;
  }
}

loadFoo();

new LoadFoo();

const bar = 2;

function computed(bar = 1) {
  return {
    [bar]: 123,
    [defaultFoo]: 234,
  }
}
computed();