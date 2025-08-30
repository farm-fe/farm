export const hello = 'hello';

const world = 'world';

export { world };

export default function sayHello() {
  console.log(hello, world);
}