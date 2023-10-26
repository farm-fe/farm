const modules = import.meta.glob('./dir/*.js', {
  query: { foo: 'bar', bar: true },
})