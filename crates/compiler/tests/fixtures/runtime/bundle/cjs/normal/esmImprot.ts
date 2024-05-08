import { name as esmNamed } from './esmExport';
import * as esmNs from './esmExport';
import esmDefault from './esmExport';
import { age as cjsNamed } from './cjsExport';
import cjsDefault from './cjsExport';
import * as cjsNs from './cjsExport';

console.log({ cjsNamed, cjsDefault, cjsNs });

console.log({ esmNamed, esmNs: esmNs, esmDefault });
