import { defineComponent } from 'vue'

defineComponent((props: {
  foo: number,
}) => { }, {})

// shouldn't be resolved
defineComponent((props: {
  foo: number,
}) => { }, {
  props: {
    bar: {
      type: String,
    },
  },
})
