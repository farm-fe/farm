import * as NamedNamespace from './exportNamed';
import { namedA, namedB, namedC } from './exportNamed';
import DefaultNamed from './exportNamed';
import Expr from './exportExpr';

import * as SameNameWithFileNamespace from './sameNameWithFile';

console.log({
  NamedNamespace,
  namedA,
  namedB,
  namedC,
  DefaultNamed,
  SameNameWithFileNamespace,
  Expr
});
