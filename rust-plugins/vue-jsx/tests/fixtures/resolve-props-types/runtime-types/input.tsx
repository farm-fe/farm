import { defineComponent } from 'vue'

type FunctionType = {
  (): void
}
type ObjectType = {
  foo: string
}
type MixedType = {
  foo: string
  (): void
}

interface FunctionInterface {
  (): void
}
interface ObjectInterface {
  foo: string
}
interface MixedInterface {
  foo: string
  (): void
}

defineComponent((props: {
  a: string
  b: number
  c: boolean
  d: object
  e: null
  f: bigint
  g: symbol
  h: any
  i: () => void
  j: new () => object
  k: string[]
  l: Array<string>
  m: [string, number]
  n: 'literal'
  o: 123
  p: 123n
  q: true
  r: FunctionType
  s: ObjectType
  t: MixedType
  u: FunctionInterface
  v: ObjectInterface
  w: MixedInterface
}) => { })

defineComponent((props: {
  function: Function,
  object: Object,
  set: Set<any>,
  map: Map<any, any>,
  weakSet: WeakSet<any>,
  weakMap: WeakMap<any, any>,
  date: Date,
  promise: Promise<any>,
  error: Error,
  regexp: RegExp,
}) => { })

defineComponent((props: {
  partial: Partial<{ foo: string }>,
  required: Required<{ foo?: string }>,
  readonly: Readonly<{ foo: string }>,
  record: Record<string, number>,
  pick: Pick<{ foo: string, bar: number }, 'foo'>,
  omit: Omit<{ foo: string, bar: number }, 'foo'>,
  instance: InstanceType<typeof String>,
}) => { })

defineComponent((props: {
  uppercase: Uppercase<'foo'>,
  lowercase: Lowercase<'FOO'>,
  capitalize: Capitalize<'foo'>,
  uncapitalize: Uncapitalize<'FOO'>,
}) => { })

defineComponent((props: {
  parameters: Parameters<() => void>,
  constructorParameters: ConstructorParameters<typeof String>,
}) => { })

defineComponent((props: {
  nonNullable: NonNullable<string | null | undefined>,
  exclude: Exclude<string | number | boolean, boolean>,
  extract: Extract<string | number | boolean, boolean>,
  omitThisParameter: OmitThisParameter<(this: string) => void>,
}) => { })
