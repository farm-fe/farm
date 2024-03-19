declare module '*.svg';
declare module '*.png';
declare module '*.css';
declare module '~react-page' {
  import type { RouteObject } from 'react-router'

  const routes: RouteObject[]
  export default routes
}

declare module 'virtual:generated-pages-react' {
  import type { RouteObject } from 'react-router'

  const routes: RouteObject[]
  export default routes
}
