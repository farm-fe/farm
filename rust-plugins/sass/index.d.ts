export interface SassOptions {
  /**
   * Add extra content to the head of each sass file, such as an @import '@/styles/variables.scss'; statement.
   */
  additionalData?: string;
}

declare const binPath: (options?: SassOptions) => [string, typeof options];
export default binPath;
