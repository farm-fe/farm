import { data } from './data';

export function Comp3() {
  const id = 'comp3';

  return {
    render: async () => {
      const { data: data1 } = await import('./data');

      const renderData = data('comp-id') + ':' + data1(id);

      const div = document.createElement('div', {});
      div.id = id;
      div.innerText = renderData;
      div.className = 'box';
      return div;
    }
  };
}
