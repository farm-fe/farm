// self accept without reload the page
import { data } from './data';
import { Comp1 } from './comp1';

async function render() {
  const root = document.getElementById('root');
  // remove all children of root
  root!.innerHTML = '';

  const renderData = data();
  const div = document.createElement('div');
  div.id = 'root-comp';
  div.innerText = renderData;
  root?.appendChild(div);

  const comps = await Promise.all([
    Comp1(),
    import('./comp2').then((mod) => mod.Comp2()),
    import('./comp3').then((mod) => mod.Comp3())
  ]);

  comps.forEach(async (comp) => {
    root?.appendChild(await comp.render());
  });
}

render();
