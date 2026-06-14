import { defineComponent } from 'vue'

const Component1 = defineComponent(() => { })

const Component2 = defineComponent(() => { }, { foo: 'bar' })

const Component3 = defineComponent(() => { }, opts)

const Component4 = defineComponent(() => { }, ...args)
