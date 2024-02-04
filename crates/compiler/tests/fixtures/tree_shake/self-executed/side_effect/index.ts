import Foo from './inner_side_effect';
import Bar from './import_side_effect';
import Goo from './write_use_side_effect_stmt';

console.log(Foo, Bar, Goo);
