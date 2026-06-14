import { defineComponent } from 'vue'

type Props = {
  foo: string
}

function scope() {
  type Props = {
    bar: number
  }

  defineComponent((props: Props) => {})
}

defineComponent((props: Props) => {})
