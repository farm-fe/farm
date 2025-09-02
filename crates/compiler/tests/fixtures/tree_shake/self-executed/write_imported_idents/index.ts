import { runtime, Camera } from '@antv/g';
import { a } from './dep';
import * as ns from './dep';
import * as ns_copy from './dep';
import * as ns2 from './dep2';

var AdvancedCamera = /*#__PURE__*/function(_Camera) {
  return _Camera;
}(Camera);

// this has side effects for ident a, so we should preserve the import statement
ns.a.Camera = AdvancedCamera;
const ns_a = ns_copy.a;
ns_a.field = 'ns_a'; // should be preserved
ns.b.field = 'b'; // should be preserved as it contains side effects

ns.c.field = 'c'; // should be removed as it is not used
ns.d.field = 'd'; // should be removed as it is not used and does not contain side effects

// we don't know the field is a or b, so we should preserve the all export of dep2
// cause if we remove the export, a runtime error will be thrown when the field cannot be found
ns2[window.document.DOCUMENT_FRAGMENT_NODE].field = '2';

console.log(runtime.AnimationTimeline, a);