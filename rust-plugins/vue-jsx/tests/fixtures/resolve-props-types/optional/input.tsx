import { defineComponent } from 'vue'

defineComponent((props: {
  foo?: number, // property
  bar?(): void, // method
  'baz'?: string, // string literal key

  untyped1?,
  untyped2?(),
}) => { })
