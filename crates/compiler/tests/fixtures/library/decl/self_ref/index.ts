import addOneClass from './exportFn';
import AddOneClass from './exportClass';

var _addClass = function addClass () {
  console.log('addClass');
  addOneClass();
}

var _AddClass = class AddClass {
  constructor() {
    console.log('addClass');
    new AddOneClass()
  }
}

