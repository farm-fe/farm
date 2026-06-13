import { defineComponent } from 'vue'
import { defaults } from './foo'

defineComponent((props: {
  foo?: string,
  bar?: number,
  baz: boolean,
  qux?(): number,
  quux?(): void,
  quuxx?: Promise<string>,
  fred?: string,
} = {
    foo: 'hi',
    qux() {
      return 1
    },
    ['quux']() { },
    async quuxx() {
      return await Promise.resolve('hi')
    },
    get fred() {
      return 'fred'
    },
  }) => { })

defineComponent((props: {
  foo?: string,
  bar?: number,
  baz: boolean,
} = { ...defaults }) => { })

defineComponent((props: {
  foo?: string,
  bar?: number,
  baz: boolean,
} = defaults) => { })

defineComponent((props: {
  foo?: () => 'string',
} = {
    ['fo' + 'o']() {
      return 'foo'
    },
  }) => { })
