import React from 'react';
import { RouterProvider } from 'react-router-dom';

import { router } from './router';

export function App() {
  return (
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
}
