import { defineComponent, type SetupContext } from 'vue'

export type Emits = { (e: 'foo' | 'bar'): void }

defineComponent((_, ctx: SetupContext<Emits>) => {})
