import bundle2_default from './default';
import * as bundle2_namespace from './namespace';
import { foo_named as bundle2_named } from './named';

console.log({ bundle2_default, bundle2_namespace, bundle2_named });

export { bundle2_default, bundle2_namespace, bundle2_named }