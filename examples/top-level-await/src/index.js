import dep1 from './dep1.js';
import sync2 from './sync2.js';

const root = document.querySelector('#root');
root.innerHTML = `<div>
dep1: ${dep1}
sync2: ${JSON.stringify(sync2)}
</div>`;
