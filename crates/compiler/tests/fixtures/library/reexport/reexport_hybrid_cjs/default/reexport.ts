export { default } from './foo.ts'; // TODO default is not work as expected, will fix it
// import './circle';

class foo {
  constructor() {
    console.log(this.constructor === foo);
  }
};

export const bar = 'foo';

export { foo };