import { products } from './products.js';
import './style.css';
const search = document.querySelector('#search');
const category = document.querySelector('#category');
function render() {
  const filtered = products.filter(product => (category.value === 'all' || product.category === category.value) && product.name.toLowerCase().includes(search.value.toLowerCase()));
  document.querySelector('#count').textContent = `${filtered.length} products`;
  const catalog = document.querySelector('#catalog');
  catalog.replaceChildren(...filtered.map(product => {
    const button = document.createElement('button');
    button.textContent = product.name;
    button.dataset.product = product.id;
    button.onclick = async () => (await import('./quote-panel.js')).showQuote(product);
    return button;
  }));
}
search.oninput = render;
category.onchange = render;
render();
