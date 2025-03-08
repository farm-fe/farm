import { Suspense, lazy, type ReactNode } from "react";


import App from './views/App';

const HomePage = lazy(() => import('./views/Home/Index'));

export default [
  // These are the same as the props you provide to <Route>
  {
    path: "/*",
    element: <App />,
    children: [
      {
        path: '*',
        element: <HomePage />
      },
    ]
  },
]
