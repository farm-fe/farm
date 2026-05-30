import { defineComponent, type SetupContext } from 'vue'

defineComponent((_, ctx: SetupContext<(e: 'foo' | 'bar') => void>) => {})

defineComponent((_, ctx: SetupContext<((e: 'foo' | 'bar') => void) | ((e: 'baz', id: number) => void)>) => {})

defineComponent((_, ctx: SetupContext<{(e: 'foo' | 'bar'): void; (e: 'baz', id: number): void;}>) => {})
