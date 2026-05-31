import { defineComponent } from 'vue'

defineComponent((props: {
  foo: number, // property
  bar(): void, // method
  'baz': string, // string literal key
  get qux(): number, // getter
  (e: 'foo'): void, // call signature
  (e: 'bar'): void,

  untyped1,
  get untyped2(),
  untyped3(),
}) => { })
