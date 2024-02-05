import { get } from './cache';

console.log(get('1').a);

const r = get('1').a;

export default r;
