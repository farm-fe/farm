import { defineComponent } from 'vue'

type T = { foo?: number, bar?: string }

defineComponent((props: Required<T>) => { })
