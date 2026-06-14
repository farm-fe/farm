import { defineComponent } from 'vue'

defineComponent(({}: {
  foo: number,
}) => { })

defineComponent(([]: {
  foo: number,
}) => { })

defineComponent(function (props: {
  foo: number,
}) { })

defineComponent(function ({}: {
  foo: number,
}) { })

defineComponent(function ([]: {
  foo: number,
}) { })
