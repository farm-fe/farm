// export { default } from './foo.ts';
// import './circle';

class foo {
  constructor() {
    console.log(this.constructor === foo);
  }
};

export const bar = 'foo';

export { foo as default };