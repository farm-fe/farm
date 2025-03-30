import { hello as world } from './export';
// import './export';

var hello = world;
function say() {
  var hello = world;
  var hello$1 = world;

  console.log(hello);
  function nested_say() {
    var hello = world;
    var hello$2 = world;

    console.log(hello);
  }
}

say();