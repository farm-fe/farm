<template>
  <main class="shell">
    <h1>Farm + Vue</h1>
    <nav>
      <RouterLink to="/">Home</RouterLink>
      <RouterLink to="/about">About</RouterLink>
    </nav>
    <RouterView />
  </main>
</template>

<style scoped>
.shell {
  display: grid;
  gap: 1rem;
  max-width: 42rem;
  margin: 2rem auto;
  padding: 2rem;
  border: 1px solid #d8f3dc;
  border-radius: 16px;
}

nav {
  display: flex;
  gap: 1rem;
}
</style>
