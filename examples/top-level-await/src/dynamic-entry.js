import { data5 } from './export-star';
import data from './import-star';

console.log('dynamic-entry', data.default, data5);

export default await new Promise(resolve => {
  setTimeout(() => {
    resolve({
      data: data.default,
      data5,
    });
  }, 1000);
});