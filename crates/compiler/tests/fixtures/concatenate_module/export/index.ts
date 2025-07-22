import { createEmotion } from './reexport';
import { createEmotion as renamedEmotion } from './reexport';
import { re, re as renamedRe, conflict as renamedConflict } from './reexport';

const conflict = 'local conflict';

console.log(createEmotion, renamedEmotion, re, renamedRe, conflict, renamedConflict);
