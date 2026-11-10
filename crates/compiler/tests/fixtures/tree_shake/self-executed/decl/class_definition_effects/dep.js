export const x = 1;

class StaticBlock {
  static { console.log('static block'); }
}
class StaticField {
  static field = console.log('static field');
}
class ComputedMethod {
  [console.log('computed method')]() {}
}
class ComputedField {
  [console.log('computed field')] = console.log('deferred instance field');
}
const Expression = class {
  static { console.log('class expression'); }
};
export class Exported {
  static field = console.log('exported class');
}
export default class {
  static #field = console.log('default class');
}
class Deferred {
  field = console.log('unused instance');
  constructor() { console.log('unused constructor'); }
  method() { console.log('unused method'); }
  static method() { console.log('unused static method'); }
}
