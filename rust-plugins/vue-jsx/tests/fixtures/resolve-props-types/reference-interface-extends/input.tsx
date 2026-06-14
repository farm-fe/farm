import { defineComponent } from 'vue'

export interface A { a(): void }
export interface B extends A { b: boolean }
interface C { c: string }
interface Aliased extends B, C { foo: number }

defineComponent((props: Aliased) => { })
