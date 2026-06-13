import { defineComponent, type SetupContext } from 'vue'

type BaseEmit = "change"
type Emit = "some" | "emit" | BaseEmit

defineComponent((_, ctx: SetupContext<{
  (e: Emit): void;
  (e: "another", val: string): void;
}>) => {})
