import { defineComponent } from 'vue'

interface Foo {
  a: string
}
interface Foo {
  b: number
}

defineComponent((props: {
  foo: Foo['a'],
  bar: Foo['b'],
}) => { })
