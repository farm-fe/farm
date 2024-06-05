import React, { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter as Router, useRoutes } from 'react-router-dom';
import './index.css';
import routes from '~react-pages';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';

const queryClient = new QueryClient();

const container = document.querySelector('#root');
const root = createRoot(container);

function App() {
  return <Suspense fallback={<p>Loading...</p>}>{useRoutes(routes)}</Suspense>;
}

root.render(
  <QueryClientProvider client={queryClient}>
    {/* <ReactQueryDevtools initialIsOpen={true} /> */}
    <Router>
      <App />
    </Router>
  </QueryClientProvider>
);
