function render(m) {
  return 'Hello, world!' + m;
}

import.meta.hot.accept((module) => {
  render(module.default);
});
