import { defineComponent } from 'vue'

type Foo = { foo: number }
type Bar = { bar: string }
type Baz = { bar: string | boolean }

defineComponent((props: { self: any } & Foo & Bar & Baz) => { })
