export interface IOptions {
  /**
   * The path to the root of the project
   */
  include?: string[];
  /**
   * exclude the path from the project
   */
  exclude?: string[];
}

declare const binPath: (options?:IOptions)=>[string, IOptions];
export default binPath;
