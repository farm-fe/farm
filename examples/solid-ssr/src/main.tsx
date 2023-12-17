import { A, RouteDefinition } from '@solidjs/router';

import { ParentProps } from 'solid-js';

function App(props: ParentProps) {
  return (
    <div>
      <h1>Server Rendering Example</h1>

      <p>
        If you check out the HTML source of this page, you'll notice that it
        already contains the HTML markup of the app that was sent from the
        server!
      </p>

      <p>
        This is great for search engines that need to index this page. It's also
        great for users because server-rendered pages tend to load more quickly
        on mobile devices and over slow networks.
      </p>

      <p>
        Another thing to notice is that when you click one of the links below
        and navigate to a different URL, then hit the refresh button on your
        browser, the server is able to generate the HTML markup for that page as
        well because you're using React Router on the server. This creates a
        seamless experience both for your users navigating around your site and
        for developers on your team who get to use the same routing library in
        both places.
      </p>

      {props.children}
    </div>
  );
}

function Layout(props: ParentProps) {
  return (
    <div>
      {/* A "layout route" is a good place to put markup you want to
          share across all the pages on your site, like navigation. */}
      <nav>
        <ul>
          <li>
            <A href="/">Home</A>
          </li>
          <li>
            <A href="/about">About</A>
          </li>
          <li>
            <A href="/dashboard">Dashboard</A>
          </li>
          <li>
            <A href="/nothing-here">Nothing Here</A>
          </li>
        </ul>
      </nav>

      <hr />

      {/* An <Outlet> renders whatever child route is currently active,
          so you can think about this <Outlet> as a placeholder for
          the child routes we defined above. */}
      {props.children}
    </div>
  );
}

function Home() {
  return (
    <div>
      <h2>Home</h2>
    </div>
  );
}

function About() {
  return (
    <div>
      <h2>About</h2>
    </div>
  );
}

function Dashboard() {
  return (
    <div>
      <h2>Dashboard</h2>
    </div>
  );
}

function NoMatch() {
  return (
    <div>
      <h2>Nothing to see here!</h2>
      <p>
        <A href="/">Go to the home page</A>
      </p>
    </div>
  );
}

export const routes = [
  {
    path: '/',
    component: App,
    children: [
      {
        path: '/',
        component: Layout,
        children: [
          {
            path: '/',
            component: Home
          },
          {
            path: 'about',
            component: About
          },
          {
            path: 'dashboard',
            component: Dashboard
          },
          {
            path: '*',
            component: NoMatch
          }
        ]
      }
    ]
  }
] satisfies RouteDefinition[];

export default App;
