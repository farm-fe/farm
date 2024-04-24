<template>
  <div>
    <nav>
      <ul>
        <li>
          <router-link to="/">Home</router-link>
        </li>
        <li>
          <router-link to="/about">About</router-link>
        </li>
        <li>
          <router-link to="/dashboard">Dashboard</router-link>
        </li>
        <li>
          <router-link to="/nothing-here">Nothing Here</router-link>
        </li>
      </ul>
    </nav>

    <hr />

    <router-view></router-view>
  </div>
</template>
