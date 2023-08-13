export interface TestBase {
  name: string
}

export interface Component {
  name: string,
  type: string
}

export const test: TestBase = {
  name: 'test'
}

export const CONSTANT = ['one', 'two'] as const

export interface WithConstant {
  constant: typeof CONSTANT[number]
}

export function method(arg: string) {
  console.log(arg)
}
