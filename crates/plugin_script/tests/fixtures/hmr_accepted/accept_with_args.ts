function render(m) {
  return 'Hello, world!' + m;
}

module.meta.hot.accept((module) => {
  render(module.default);
});
