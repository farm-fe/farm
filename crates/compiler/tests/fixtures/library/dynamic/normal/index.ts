import('./dynamic').then(res => res.default());

const foo = () => import('./dynamic');

function loader(m: any) {}

loader(import('./dynamic'));

import('./dynamic');

const data = {
  foo: import('./dynamic'),
}

{
  import('./same-name/dynamic');
}