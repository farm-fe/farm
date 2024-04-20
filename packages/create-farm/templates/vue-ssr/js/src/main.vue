<template>
  <div>
    <h1>Server Rendering Example</h1>

    <p>
      If you check out the HTML source of this page, you'll notice that it
      already contains the HTML markup of the app that was sent from the server!
    </p>

    <p>
      This is great for search engines that need to index this page. It's also
      great for users because server-rendered pages tend to load more quickly on
      mobile devices and over slow networks.
    </p>

    <p>
      Another thing to notice is that when you click one of the links below and
      navigate to a different URL, then hit the refresh button on your browser,
      the server is able to generate the HTML markup for that page as well
      because you're using React Router on the server. This creates a seamless
      experience both for your users navigating around your site and for
      developers on your team who get to use the same routing library in both
      places.
    </p>
    <router-view></router-view>
  </div>
</template>
