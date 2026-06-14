import { defineComponent } from 'vue'

export interface Aliased { foo: number }

defineComponent((props: Aliased) => { })
