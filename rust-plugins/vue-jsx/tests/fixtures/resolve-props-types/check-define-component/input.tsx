function scope() {
  function defineComponent() {}

  defineComponent((props: { msg: string }) => {})
}

defineComponent((props: { msg: string }) => {})
