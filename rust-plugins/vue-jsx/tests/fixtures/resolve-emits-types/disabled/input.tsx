import { defineComponent, type SetupContext } from 'vue'

defineComponent((_, ctx: SetupContext<(e: 'foo' | 'bar') => void>) => {})
