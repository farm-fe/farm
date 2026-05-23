import type { UserConfig } from "@farmfe/core"
export interface IPluginOptions {
  isBuild?: boolean
  compilerConfig?: UserConfig['compilation']
}
