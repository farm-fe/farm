import { defineComponent } from 'vue'

export type Aliased = { foo: number }

defineComponent((props: Aliased) => { })
