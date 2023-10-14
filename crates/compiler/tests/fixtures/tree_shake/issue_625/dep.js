import { getTickMethod, registerTickMethod } from './register';

registerTickMethod('xxx', () => console.log('xxx'));

export { getTickMethod, registerTickMethod };