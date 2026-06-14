import { defineComponent } from 'vue'

type A = (string | number)[]
type AA = Array<string>
type T = [1, 'foo']
type TT = [foo: 1, bar: 'foo']
type Optional = [1?]

defineComponent((props: {
  foo: A[number],
  bar: AA[number],
  tuple: T[number],
  tuple0: T[0],
  tuple1: T[1],
  namedTuple: TT[number],
  optional: Optional[0],
}) => { })
