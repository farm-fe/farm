import { defineComponent } from 'vue'

type T = { foo: number, bar: string, baz: boolean }
type K = 'foo' | 'bar'

defineComponent((props: Omit<T, K>) => { })
