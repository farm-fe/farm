import { defineComponent } from 'vue'

type K = 'foo' | 'bar'
interface T { foo: string, bar: number }
interface Foo { foo: T[string] }
interface Bar { bar: string }
interface S { foo: Foo, bar: Bar }

defineComponent((props: S[K]) => { })
