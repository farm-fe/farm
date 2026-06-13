import { defineComponent, type SetupContext } from 'vue'

interface Base { (e: 'foo'): void }
interface Emits extends Base { (e: 'bar'): void }

defineComponent((_, ctx: SetupContext<Emits>) => {})
