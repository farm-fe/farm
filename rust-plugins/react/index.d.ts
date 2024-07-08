export interface ReactConfig {
  /**
   * Replace the function used when compiling JSX expressions.
   *
   * Defaults to `React.createElement`.
   */
  pragma?: string;
  /**
   * Replace the component used when compiling JSX fragments.
   *
   * Defaults to `React.Fragment`
   */
  pragmaFrag?: string;
  /**
   * Toggles whether or not to throw an error if a XML namespaced tag name is used. For example:
   * `<f:image />`
   *
   * Though the JSX spec allows this, it is disabled by default since React's
   * JSX does not currently have support for it.
   *
   */
  throwIfNamespace?: boolean;
  /**
   * Toggles plugins that aid in development, such as @swc/plugin-transform-react-jsx-self
   * and @swc/plugin-transform-react-jsx-source.
   *
   * Defaults to `false`,
   *
   */
  development?: boolean;
  /**
   * Use `Object.assign()` instead of `_extends`. Defaults to false.
   * @deprecated
   */
  useBuiltins?: boolean;

  /**
   * Enable fast refresh feature for React app
   */
  refresh?: boolean;

  /**
   * jsx runtime
   */
  runtime?: "automatic" | "classic";

  /**
   * Declares the module specifier to be used for importing the `jsx` and `jsxs` factory functions when using `runtime` 'automatic'
   */
  importSource?: string;
}
declare const binPath: (options?:ReactConfig)=>[string, ReactConfig];
export default binPath;
