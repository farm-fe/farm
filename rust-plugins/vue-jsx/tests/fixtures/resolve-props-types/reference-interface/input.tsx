import { defineComponent } from 'vue'

interface Aliased { foo: number }

defineComponent((props: Aliased) => { })

defineComponent((props: (Aliased)) => { })
