export type ImportNameAlias = [string, string]
export type InlinePreset = { from: string, imports: string[] }
type Preset = string
  | InlinePreset
  | Record<string, (string | ImportNameAlias)[]>

export interface IPluginOptions {
  /**
   * The directories to search for apis.
   * @default process.cwd()
   * @example
   * dirs: ["src/addons"]
   */
  dirs?: string[];
  /**
   * generate d.ts file.
   * @default true
   * @example
   * dts: "src/types/auto-import.d.ts"
   */
  dts?: boolean | string;
  /**
   * The ignore patterns.
   * @example
   * ignore: ["useFoo","^my*"]
   */
  ignore?: string[];
  /**
   * The presets of the project.
   * @example
   * presets: [
   *  "react",
   *  {
   *    from: "react",
   *    imports: ["useState","useEffect"]
   *  }, 
   *  {
   *    "react-router-dom": 
   *      [
   *        "useHistory",
   *        ["useParams","useReactPrams"]
   *     ]
   *  }
   * ]
   */
  presets?: Preset[];
  include?: string[];
  exclude?: string[];
  injectAtEnd?: boolean;
}
