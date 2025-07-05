import { sayZoo } from "./zoo";

export const hello = "hello";
export const world = "world";

export function sayHello() {
  console.log(hello, world);
  sayZoo();
}