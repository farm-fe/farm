import { sayHello } from "./export";

export const zoo = "zoo";

function sayZoo() {
  console.log(zoo);
  sayHello();
}
export { sayZoo };