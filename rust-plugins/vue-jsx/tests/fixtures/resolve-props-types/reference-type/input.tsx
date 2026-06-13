import { defineComponent } from 'vue'

type Aliased = { foo: number }

defineComponent((props: Aliased) => { })
