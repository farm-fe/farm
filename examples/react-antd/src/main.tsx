import React from 'react';
import { AntdLayout } from './antd-layout';

import { createBrowserRouter, RouterProvider } from 'react-router-dom';

import url from '../assets/plugin.svg?inline';

const router = createBrowserRouter(
  [
    {
      path: '/',
      element: <AntdLayout />,
      children: [
        {
          path: '1',
          element: (
            <div>
              1111 <img src={url} width={200} />
            </div>
          )
        },
        {
          path: '2',
          element: <div>2222</div>
        },
        {
          path: '3',
          element: <div>3333</div>
        }
      ]
    }
  ],
  {
    basename: '/admin'
  }
);

export function App() {
  return (
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
}
