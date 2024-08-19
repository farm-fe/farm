import React from 'react';
import { AntdLayout } from './antd-layout';

import { createBrowserRouter } from 'react-router-dom';

import url from '../assets/plugin.svg?inline';
import { getRoutes } from './extra-routes';

export const router = createBrowserRouter(
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
        ...getRoutes(),
      ]
    }
  ],
  {
    basename: '/admin'
  }
);
