import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { HelmetProvider } from 'react-helmet-async';
import { ConfigProvider } from "antd";
import { ProConfigProvider } from "@ant-design/pro-components";
import { createHashHistory, createRouter, RouterProvider } from '@tanstack/react-router';

// Import the generated route tree
import { routeTree } from './routeTree.gen'

const history = createHashHistory();
// Create a new router instance
const router = createRouter({ routeTree, history: history })

const container = document.getElementById('root') as HTMLElement;
const root = createRoot(container);

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

root.render(<StrictMode>
  <HelmetProvider>
    <ConfigProvider
      getTargetContainer={() => container || document.body}
    >
      <ProConfigProvider hashed={false}>
        <RouterProvider router={router} />
      </ProConfigProvider>
    </ConfigProvider>
  </HelmetProvider>
</StrictMode>);
