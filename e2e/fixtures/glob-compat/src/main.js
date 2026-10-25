const eager = import.meta.glob('./files/*.{js,ts}', { eager: true, import: 'default' });
const lazy = import.meta.glob(['!./files/*.skip.js', './files/*.js']);
const query = import.meta.glob('./files/a.js', { query: '?compat=1', eager: true, import: 'name' });
const based = import.meta.glob('./*.js', { base: './files', eager: true, import: 'default' });
const rooted = import.meta.glob('/src/files/a.js', { eager: true, import: 'default' });
const alias = import.meta.glob('@files/a.js', { eager: true, import: 'default' });
const exhaustive = import.meta.glob('./files/*.js', { exhaustive: true, eager: true, import: 'default' });
async function paint() {
  const values = await Promise.all(Object.values(lazy).map(load => load().then(module => module.default)));
  const state = { eager, lazyKeys: Object.keys(lazy), lazyValues: values, query, based, rooted, alias, exhaustive };
  globalThis.__globCompat = state;
  document.querySelector('#result').textContent = JSON.stringify(state);
}
void paint();
if (import.meta.hot) import.meta.hot.accept();
